//! Governed egui-managed PDF uploads for the alternate compositor.
use eframe::{
    egui,
    glow::{self, HasContext},
};
use std::sync::{Arc, Mutex, Weak};
use yamantaka_domain::{
    Failure, RenderedPage, Result,
    memory::{ByteLease, Cost, MemoryCategory, MemoryGovernor, MemoryRepresentation},
};

struct Retirement {
    frame: u64,
    lease: ByteLease,
}
type Retirements = Arc<Mutex<Vec<Retirement>>>;

pub(super) struct Texture {
    handle: Option<egui::TextureHandle>,
    lease: Option<ByteLease>,
    retirements: Retirements,
    context: egui::Context,
}
impl Texture {
    pub fn id(&self) -> egui::TextureId {
        self.handle.as_ref().unwrap().id()
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        // Queue egui's deletion before fencing a subsequent painted frame.
        self.handle.take();
        self.retirements
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(Retirement {
                frame: self.context.cumulative_frame_nr(),
                lease: self.lease.take().unwrap(),
            });
        self.context.request_repaint();
    }
}
struct CopyLease {
    image: Weak<egui::ColorImage>,
    _lease: ByteLease,
}
struct Fence {
    sync: glow::Fence,
    leases: Vec<ByteLease>,
}

pub(super) struct Presenter {
    governor: MemoryGovernor,
    retirements: Retirements,
    copies: Vec<CopyLease>,
    fences: Vec<Fence>,
    shutdown_leases: Vec<ByteLease>,
    last_frame: Option<u64>,
    failed: bool,
    retired: u64,
}
impl Presenter {
    pub fn new(governor: MemoryGovernor) -> Self {
        Self {
            governor,
            retirements: Default::default(),
            copies: Vec::new(),
            fences: Vec::new(),
            shutdown_leases: Vec::new(),
            last_frame: None,
            failed: false,
            retired: 0,
        }
    }
    pub fn upload(&mut self, ctx: &egui::Context, page: &RenderedPage) -> Result<Texture> {
        let cap = ctx
            .input(|input| input.max_texture_side)
            .min(yamantaka_domain::MAX_RENDER_DIMENSION as usize);
        let bytes = (page.width as usize)
            .checked_mul(page.height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| Failure::Invalid("Managed page dimensions overflow".into()))?;
        if page.width == 0
            || page.height == 0
            || page.width as usize > cap
            || page.height as usize > cap
            || bytes != page.pixels().len()
        {
            return Err(Failure::Invalid(
                "Managed page pixels exceed their dimensions or device limit".into(),
            ));
        }
        let category = if page.key.region.is_some() {
            MemoryCategory::Tiles
        } else {
            MemoryCategory::Pages
        };
        let mut leases = self
            .governor
            .try_reserve(&[
                Cost::new(category, MemoryRepresentation::Gpu, bytes),
                Cost::new(category, MemoryRepresentation::Transient, bytes),
            ])
            .map_err(crate::page_presenter::memory_failure)?;
        let copy = leases.pop().unwrap();
        let gpu = leases.pop().unwrap();
        let image = Arc::new(egui::ColorImage::from_rgba_unmultiplied(
            [page.width as usize, page.height as usize],
            page.pixels(),
        ));
        self.copies.push(CopyLease {
            image: Arc::downgrade(&image),
            _lease: copy,
        });
        let handle = ctx.load_texture(
            "pdf-page",
            egui::ImageData::Color(image),
            egui::TextureOptions::LINEAR,
        );
        ctx.request_repaint();
        Ok(Texture {
            handle: Some(handle),
            lease: Some(gpu),
            retirements: self.retirements.clone(),
            context: ctx.clone(),
        })
    }
    /// Called only from the visible root UI, with eframe's current GL context.
    pub fn begin_frame(&mut self, ctx: &egui::Context, gl: &glow::Context) -> Result<()> {
        let frame = ctx.cumulative_frame_nr();
        if self.last_frame == Some(frame) || self.failed {
            return Ok(());
        }
        self.last_frame = Some(frame);
        self.copies.retain(|copy| copy.image.strong_count() != 0);
        let mut ready = Vec::new();
        {
            let mut queue = self
                .retirements
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let mut index = 0;
            while index < queue.len() {
                if queue[index].frame < frame {
                    ready.push(queue.swap_remove(index).lease);
                } else {
                    index += 1;
                }
            }
        }
        if !ready.is_empty() {
            // GL commands are ordered after the previous paint and texture frees.
            let sync = unsafe { gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0) };
            match sync {
                Ok(sync) => {
                    self.fences.push(Fence {
                        sync,
                        leases: ready,
                    });
                    unsafe {
                        gl.flush();
                    }
                }
                Err(_) => {
                    self.shutdown_leases.extend(ready);
                    self.failed = true;
                    return Err(Failure::Pdf(
                        "Managed page retirement fence could not be created".into(),
                    ));
                }
            }
        }
        let mut failed = false;
        self.fences.retain(|fence| {
            // Zero timeout polls; ordinary reading never waits for the GPU.
            match unsafe { gl.client_wait_sync(fence.sync, 0, 0) } {
                glow::ALREADY_SIGNALED | glow::CONDITION_SATISFIED => {
                    unsafe {
                        gl.delete_sync(fence.sync);
                    }
                    self.retired += fence.leases.len() as u64;
                    false
                }
                glow::WAIT_FAILED => {
                    failed = true;
                    true
                }
                _ => true,
            }
        });
        self.failed |= failed;
        if failed {
            return Err(Failure::Pdf("Managed page retirement fence failed".into()));
        }
        if !self.settled() {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        Ok(())
    }
    pub fn settled(&self) -> bool {
        self.copies.is_empty()
            && self.fences.is_empty()
            && self
                .retirements
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .is_empty()
    }
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({"settled":self.settled(),"pendingCopies":self.copies.len(),"pendingFences":self.fences.len(),"retiredTextures":self.retired,"failed":self.failed})
    }
    pub fn shutdown(&mut self, gl: &glow::Context) {
        for fence in self.fences.drain(..) {
            unsafe {
                gl.delete_sync(fence.sync);
            }
            self.shutdown_leases.extend(fence.leases);
        }
        // eframe destroys its painter just after on_exit; retain charges until
        // this application is dropped, after that renderer teardown.
        self.shutdown_leases.extend(
            self.retirements
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .drain(..)
                .map(|item| item.lease),
        );
    }
}
