//! Native WGPU page uploads with explicit admission and fence-bound ownership.
//!
//! This path avoids egui's `ColorImage` CPU copy. The governor covers the RGBA
//! texture payload and the aligned upload buffer, not opaque driver allocations.
//! Glow uploads and device-loss reconstruction are separate integration work.
use eframe::{egui, egui_wgpu::RenderState, wgpu};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use yamantaka_domain::{
    Failure, MAX_RENDER_DIMENSION, RenderedPage, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryError, MemoryGovernor, MemoryRepresentation},
};

const FENCE_CHECK_INTERVAL: Duration = Duration::from_millis(16);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UploadLayout {
    row_bytes: u32,
    padded_row_bytes: u32,
    texture_bytes: usize,
    staging_bytes: usize,
}

impl UploadLayout {
    fn new(width: u32, height: u32, dimension_limit: u32, buffer_limit: u64) -> Result<Self> {
        if width == 0 || height == 0 || width > dimension_limit || height > dimension_limit {
            return Err(Failure::Invalid(
                "Page texture exceeds device dimensions".into(),
            ));
        }
        let row_bytes = width.checked_mul(4).ok_or_else(layout_overflow)?;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = row_bytes
            .checked_add(alignment - 1)
            .map(|bytes| bytes / alignment * alignment)
            .ok_or_else(layout_overflow)?;
        let texture_bytes = (row_bytes as usize)
            .checked_mul(height as usize)
            .ok_or_else(layout_overflow)?;
        let staging_bytes = (padded_row_bytes as usize)
            .checked_mul(height as usize)
            .ok_or_else(layout_overflow)?;
        if staging_bytes as u64 > buffer_limit {
            return Err(Failure::Invalid(
                "Page upload exceeds the device buffer limit".into(),
            ));
        }
        Ok(Self {
            row_bytes,
            padded_row_bytes,
            texture_bytes,
            staging_bytes,
        })
    }

    fn costs(self, category: MemoryCategory) -> [Cost; 2] {
        [
            Cost::new(category, MemoryRepresentation::Gpu, self.texture_bytes),
            Cost::new(
                category,
                MemoryRepresentation::Transient,
                self.staging_bytes,
            ),
        ]
    }
}

fn layout_overflow() -> Failure {
    Failure::Invalid("Page upload layout overflow".into())
}

fn memory_failure(error: MemoryError) -> Failure {
    match error {
        MemoryError::BudgetDenied { .. } => Failure::MemoryBudget,
        other => Failure::Invalid(other.to_string()),
    }
}

// Field order drops the physical resource before its accounting lease.
struct ResidentTexture {
    texture: wgpu::Texture,
    _lease: ByteLease,
}

struct UploadBuffer {
    buffer: wgpu::Buffer,
    _lease: ByteLease,
}

impl Drop for ResidentTexture {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}

impl Drop for UploadBuffer {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}

type RetirementQueue = Arc<Mutex<Vec<egui::TextureId>>>;

/// Unique logical ownership of a registered page texture.
///
/// Drop queues retirement for the next frame boundary; it never frees a texture
/// which the current frame may still reference. IDs become invalid after the
/// presenter's `shutdown`, which must run only after the final frame is painted.
pub struct PageTexture {
    id: egui::TextureId,
    retirements: RetirementQueue,
    context: egui::Context,
}

impl PageTexture {
    pub fn id(&self) -> egui::TextureId {
        self.id
    }
}

impl Drop for PageTexture {
    fn drop(&mut self) {
        self.retirements
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(self.id);
        // One wake-up initiates retirement. Idle presenters do not repaint.
        self.context.request_repaint();
    }
}

pub struct PagePresenter {
    render: RenderState,
    governor: MemoryGovernor,
    context: egui::Context,
    resident: HashMap<egui::TextureId, ResidentTexture>,
    retirements: RetirementQueue,
    pending_fences: Arc<AtomicUsize>,
    fence_deadline: Option<Instant>,
    last_frame: Option<u64>,
    closed: bool,
}

impl PagePresenter {
    pub fn new(render: RenderState, governor: MemoryGovernor, context: egui::Context) -> Self {
        Self {
            render,
            governor,
            context,
            resident: HashMap::new(),
            retirements: Arc::new(Mutex::new(Vec::new())),
            pending_fences: Arc::new(AtomicUsize::new(0)),
            fence_deadline: None,
            last_frame: None,
            closed: false,
        }
    }

    pub fn upload(&mut self, page: &RenderedPage) -> Result<PageTexture> {
        if self.closed {
            return Err(Failure::Invalid("Page presenter is closed".into()));
        }
        let device_limits = self.render.device.limits();
        let dimension_limit = self
            .render
            .adapter
            .limits()
            .max_texture_dimension_2d
            .min(device_limits.max_texture_dimension_2d)
            .min(MAX_RENDER_DIMENSION);
        let layout = UploadLayout::new(
            page.width,
            page.height,
            dimension_limit,
            device_limits.max_buffer_size,
        )?;
        if page.pixels().len() != layout.texture_bytes {
            return Err(Failure::Invalid(
                "Page pixels do not match the upload dimensions".into(),
            ));
        }
        let category = if page.key.region.is_some() {
            MemoryCategory::Tiles
        } else {
            MemoryCategory::Pages
        };
        // Atomic admission happens before either physical allocation.
        let mut reservation = self
            .governor
            .try_reserve(&layout.costs(category))
            .map_err(memory_failure)?;
        let staging_lease = reservation.pop().expect("two admitted upload costs");
        let texture_lease = reservation.pop().expect("two admitted upload costs");
        let staging = self.render.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Yamantaka tracked page upload"),
            size: layout.staging_bytes as u64,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });
        {
            let mut mapped = staging.slice(..).get_mapped_range_mut().map_err(|error| {
                Failure::Pdf(format!("Page upload buffer mapping failed: {error}"))
            })?;
            let source_stride = layout.row_bytes as usize;
            let destination_stride = layout.padded_row_bytes as usize;
            for (row, source) in page.pixels().chunks_exact(source_stride).enumerate() {
                let start = row * destination_stride;
                mapped
                    .slice(start..start + source_stride)
                    .copy_from_slice(source);
            }
        }
        staging.unmap();
        let size = wgpu::Extent3d {
            width: page.width,
            height: page.height,
            depth_or_array_layers: 1,
        };
        let texture = self.render.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Yamantaka tracked page texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut encoder =
            self.render
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Yamantaka tracked page copy"),
                });
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.padded_row_bytes),
                    rows_per_image: Some(page.height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
        self.render.queue.submit([encoder.finish()]);
        self.retain_until_fence(UploadBuffer {
            buffer: staging,
            _lease: staging_lease,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let id = self.render.renderer.write().register_native_texture(
            &self.render.device,
            &view,
            wgpu::FilterMode::Linear,
        );
        self.resident.insert(
            id,
            ResidentTexture {
                texture,
                _lease: texture_lease,
            },
        );
        Ok(PageTexture {
            id,
            retirements: self.retirements.clone(),
            context: self.context.clone(),
        })
    }

    /// Call before drawing the new frame. Repeated egui layout passes are ignored.
    pub fn begin_frame(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        let frame = self.context.cumulative_frame_nr();
        if self.last_frame == Some(frame) {
            return Ok(());
        }
        self.last_frame = Some(frame);
        let ids = std::mem::take(
            &mut *self
                .retirements
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
        self.retire(ids);
        if self.pending_fences.load(Ordering::Acquire) != 0 {
            self.render
                .device
                .poll(wgpu::PollType::Poll)
                .map_err(|error| Failure::Pdf(format!("Page GPU fence polling failed: {error}")))?;
            if self.pending_fences.load(Ordering::Acquire) != 0 {
                if self
                    .fence_deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
                {
                    return Err(Failure::Pdf(
                        "Page GPU fence timed out; pending resources remain retained".into(),
                    ));
                }
                self.context.request_repaint_after(FENCE_CHECK_INTERVAL);
            } else {
                self.fence_deadline = None;
            }
        }
        Ok(())
    }

    fn retire(&mut self, ids: impl IntoIterator<Item = egui::TextureId>) {
        let mut resources = Vec::new();
        {
            let mut renderer = self.render.renderer.write();
            for id in ids {
                if let Some(resource) = self.resident.remove(&id) {
                    renderer.free_texture(&id);
                    resources.push(resource);
                }
            }
        }
        if !resources.is_empty() {
            // The empty submission is ordered after the final egui draw using
            // these IDs. Removing native registrations does not destroy textures.
            self.render.queue.submit(std::iter::empty());
            self.retain_until_fence(resources);
        }
    }

    fn retain_until_fence<T: Send + 'static>(&mut self, resources: T) {
        let pending = self.pending_fences.clone();
        if pending.fetch_add(1, Ordering::AcqRel) == 0 {
            self.fence_deadline = Some(Instant::now() + SHUTDOWN_TIMEOUT);
        }
        self.render.queue.on_submitted_work_done(move || {
            drop(resources);
            pending.fetch_sub(1, Ordering::AcqRel);
        });
        self.context.request_repaint_after(FENCE_CHECK_INTERVAL);
    }

    /// Retire all registrations after the final frame, then wait for their fence.
    /// A timeout leaves callback-owned resources accounted until actual completion.
    pub fn shutdown(&mut self) -> Result<()> {
        self.closed = true;
        self.retirements
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        let ids: Vec<_> = self.resident.keys().copied().collect();
        self.retire(ids);
        if self.pending_fences.load(Ordering::Acquire) != 0 {
            self.render
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(SHUTDOWN_TIMEOUT),
                })
                .map_err(|error| {
                    Failure::Pdf(format!("Page GPU shutdown fence failed: {error}"))
                })?;
        }
        Ok(())
    }
}

impl Drop for PagePresenter {
    fn drop(&mut self) {
        // Explicit shutdown reports errors; this fallback still preserves leases
        // in queue callbacks if a device does not finish within the timeout.
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yamantaka_domain::memory::GlobalMemoryLimits;

    #[test]
    fn upload_layout_preserves_tight_rgba_and_aligns_every_staging_row() {
        let layout = UploadLayout::new(65, 3, 16_384, u64::MAX).unwrap();
        assert_eq!(layout.row_bytes, 260);
        assert_eq!(layout.padded_row_bytes, 512);
        assert_eq!(layout.texture_bytes, 780);
        assert_eq!(layout.staging_bytes, 1_536);
        let aligned = UploadLayout::new(64, 3, 16_384, u64::MAX).unwrap();
        assert_eq!(aligned.texture_bytes, aligned.staging_bytes);
    }

    #[test]
    fn invalid_dimensions_overflow_and_device_buffer_limit_fail_before_allocation() {
        assert!(UploadLayout::new(0, 1, 16_384, u64::MAX).is_err());
        assert!(UploadLayout::new(1, 0, 16_384, u64::MAX).is_err());
        assert!(UploadLayout::new(4_097, 1, 4_096, u64::MAX).is_err());
        assert!(UploadLayout::new(u32::MAX, 1, u32::MAX, u64::MAX).is_err());
        assert!(UploadLayout::new(1, 2, 16_384, 511).is_err());
        assert_eq!(
            UploadLayout::new(1, 2, 16_384, 512).unwrap().staging_bytes,
            512
        );
    }

    #[test]
    fn atomic_upload_admission_includes_live_cpu_and_row_padding() {
        let governor = MemoryGovernor::new(GlobalMemoryLimits {
            total: 527,
            pages: 527,
            ..GlobalMemoryLimits::default()
        });
        let cpu = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Pages,
                MemoryRepresentation::Cpu,
                8,
            ))
            .unwrap();
        let layout = UploadLayout::new(1, 2, 16_384, u64::MAX).unwrap();
        assert!(
            governor
                .try_reserve(&layout.costs(MemoryCategory::Pages))
                .is_err()
        );
        assert_eq!(governor.snapshot().usage.total, 8);
        governor.set_limits(GlobalMemoryLimits {
            total: 528,
            pages: 528,
            ..GlobalMemoryLimits::default()
        });
        let upload = governor
            .try_reserve(&layout.costs(MemoryCategory::Pages))
            .unwrap();
        assert_eq!(governor.snapshot().usage.gpu, 8);
        assert_eq!(governor.snapshot().usage.transient, 512);
        assert_eq!(governor.snapshot().usage.pages, 528);
        drop(upload);
        assert_eq!(governor.snapshot().usage.total, 8);
        drop(cpu);
        assert_eq!(governor.snapshot().usage.total, 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn native_dx12_upload_readback_and_fenced_retirement() {
        use eframe::egui_wgpu::{Renderer, RendererOptions, SurfaceConfig};
        use yamantaka_domain::{RenderKey, memory::TrackedBytesBuilder};

        // A real headless DX12 device is required. Adapter failures fail the test,
        // rather than turning a missing native check into a silent success.
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .expect("a native DX12 adapter is required for this test");
        let info = adapter.get_info();
        assert_eq!(info.backend, wgpu::Backend::Dx12);
        println!(
            "Native page upload adapter: {} ({:?})",
            info.name, info.device_type
        );
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Yamantaka native page presenter test"),
            ..Default::default()
        }))
        .expect("create native DX12 device");
        let target_format = wgpu::TextureFormat::Rgba8Unorm;
        let renderer = Arc::new(egui::epaint::mutex::RwLock::new(Renderer::new(
            &device,
            target_format,
            RendererOptions::PREDICTABLE,
        )));
        let render = RenderState {
            adapter: adapter.clone(),
            available_adapters: vec![adapter],
            instance,
            device,
            queue,
            target_format,
            renderer,
            surface_config: SurfaceConfig::LOW_LATENCY,
        };
        let governor = MemoryGovernor::default();
        let context = egui::Context::default();
        let mut presenter = PagePresenter::new(render.clone(), governor.clone(), context.clone());
        let layout = UploadLayout::new(65, 3, 16_384, u64::MAX).unwrap();
        let mut pixels =
            TrackedBytesBuilder::try_zeroed(&governor, MemoryCategory::Pages, layout.texture_bytes)
                .unwrap();
        for (index, pixel) in pixels.as_mut_slice().chunks_exact_mut(4).enumerate() {
            let x = (index % 65) as u8;
            let y = (index / 65) as u8;
            pixel.copy_from_slice(&[x, y, x.wrapping_add(y), 255]);
        }
        let page = RenderedPage::from_pixels(
            RenderKey {
                document_id: "synthetic-native-upload".into(),
                page: 0,
                region: None,
                physical_width: 65,
                physical_height: 3,
                rotation: 0,
                generation: 1,
            },
            pixels.finish(),
        )
        .unwrap();
        let texture = presenter.upload(&page).unwrap();
        let id = texture.id();
        assert!(render.renderer.read().texture(&id).is_some());
        assert_eq!(governor.snapshot().usage.cpu, 780);
        assert_eq!(governor.snapshot().usage.gpu, 780);
        assert_eq!(governor.snapshot().usage.transient, 1_536);

        // Diagnostic readback is also admitted before its physical allocation.
        let readback_lease = governor
            .try_reserve_one(Cost::new(
                MemoryCategory::Other,
                MemoryRepresentation::Transient,
                layout.staging_bytes,
            ))
            .unwrap();
        let readback = UploadBuffer {
            buffer: render.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Yamantaka tracked native readback test"),
                size: layout.staging_bytes as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            _lease: readback_lease,
        };
        let mut encoder = render
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &presenter.resident.get(&id).unwrap().texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.padded_row_bytes),
                    rows_per_image: Some(3),
                },
            },
            wgpu::Extent3d {
                width: 65,
                height: 3,
                depth_or_array_layers: 1,
            },
        );
        render.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        readback
            .buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                sender.send(result).unwrap();
            });
        let wait = || {
            render
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(SHUTDOWN_TIMEOUT),
                })
                .expect("native GPU fence completes")
        };
        wait();
        receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap();
        {
            let mapped = readback.buffer.slice(..).get_mapped_range().unwrap();
            for (actual, expected) in mapped
                .chunks_exact(layout.padded_row_bytes as usize)
                .zip(page.pixels().chunks_exact(layout.row_bytes as usize))
            {
                assert_eq!(&actual[..layout.row_bytes as usize], expected);
            }
        }
        readback.buffer.unmap();
        assert_eq!(governor.snapshot().usage.transient, layout.staging_bytes);
        assert_eq!(governor.snapshot().usage.pages, 1_560);
        drop(readback);
        assert_eq!(governor.snapshot().usage.transient, 0);

        drop(texture);
        assert_eq!(governor.snapshot().usage.gpu, 780);
        assert!(render.renderer.read().texture(&id).is_some());
        // Separate retirement from polling here to observe the callback-owned
        // lease even when this small GPU copy has already physically completed.
        presenter.retire([id]);
        assert!(render.renderer.read().texture(&id).is_none());
        assert_eq!(governor.snapshot().usage.gpu, 780);
        assert_eq!(presenter.pending_fences.load(Ordering::Acquire), 1);
        wait();
        assert_eq!(governor.snapshot().usage.gpu, 0);
        assert_eq!(governor.snapshot().usage.cpu, 780);
        presenter.begin_frame().unwrap();

        // A second layout pass must not invalidate an ID used by this frame.
        let second = presenter.upload(&page).unwrap();
        let second_id = second.id();
        drop(second);
        presenter.begin_frame().unwrap();
        assert!(render.renderer.read().texture(&second_id).is_some());
        let mut output = context.run_ui(egui::RawInput::default(), |_| {});
        // This headless layout pass advances the frame counter without painting
        // egui's unrelated font atlas. Page pixels were read back from real DX12.
        output.textures_delta.clear();
        presenter.begin_frame().unwrap();
        assert!(render.renderer.read().texture(&second_id).is_none());
        wait();
        assert_eq!(governor.snapshot().usage.total, 780);

        // Denied joint admission neither changes the ledger nor registers an ID.
        governor.set_limits(GlobalMemoryLimits {
            total: 3_095,
            pages: 3_095,
            ..GlobalMemoryLimits::default()
        });
        let next_id = egui::TextureId::User(2);
        assert!(render.renderer.read().texture(&next_id).is_none());
        assert!(matches!(
            presenter.upload(&page),
            Err(Failure::MemoryBudget)
        ));
        assert!(presenter.resident.is_empty());
        assert!(render.renderer.read().texture(&next_id).is_none());
        assert_eq!(governor.snapshot().usage.total, 780);
        governor.set_limits(GlobalMemoryLimits::default());
        let last = presenter.upload(&page).unwrap();
        assert_eq!(last.id(), next_id);
        // Shutdown also retires a still-live logical handle after the final draw.
        presenter.shutdown().unwrap();
        assert_eq!(governor.snapshot().usage.gpu, 0);
        assert_eq!(governor.snapshot().usage.transient, 0);
        drop(last);
        drop(page);
        assert_eq!(governor.snapshot().usage.total, 0);
    }
}
