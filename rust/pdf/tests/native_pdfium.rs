use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use yamantaka_domain::{Cancellation, Failure, PdfEngine, Priority, RenderKey};
use yamantaka_pdf::{MemoryLimits, PdfWorker};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn engine() -> PdfWorker {
    PdfWorker::start(
        &root().join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits::default(),
    )
    .expect("Run tool/fetch_rust_pdfium.ps1 before native acceptance tests")
}
fn fixture(name: &str) -> PathBuf {
    root().join("build/rust-fixtures").join(name)
}
fn key(id: &str, page: u32, generation: u64) -> RenderKey {
    RenderKey {
        document_id: id.into(),
        page,
        region: None,
        physical_width: 600,
        physical_height: 800,
        rotation: 0,
        generation,
    }
}
fn hash(path: &std::path::Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}

#[test]
fn native_text_vectors_scans_rotation_password_damage_and_5000_pages() {
    let engine = engine();
    let cancel = Cancellation::default();
    for (id, name, pages) in [
        ("text", "textual.pdf", 2),
        ("scan", "scanned.pdf", 1),
        ("rotate", "rotated.pdf", 2),
        ("long", "5000-pages.pdf", 5000),
    ] {
        let path = fixture(name);
        let before = hash(&path);
        let info = engine
            .open(id.into(), path.clone(), None, cancel.clone())
            .unwrap();
        assert_eq!(info.page_sizes.len(), pages);
        let page = engine
            .render(key(id, 0, 1), Priority::Visible, cancel.clone())
            .unwrap();
        assert_eq!(page.pixels().len(), 600 * 800 * 4);
        if id == "text" {
            assert!(
                engine
                    .text(id.into(), 0, cancel.clone())
                    .unwrap()
                    .contains("Yamantaka Rust")
            );
            assert_ne!(info.page_sizes[0], info.page_sizes[1]);
        }
        if id == "scan" {
            assert!(
                engine
                    .text(id.into(), 0, cancel.clone())
                    .unwrap()
                    .is_empty()
            );
            assert!(page.pixels().chunks_exact(4).any(|pixel| pixel[0] < 100));
            let center = (400 * 600 + 300) * 4;
            assert_eq!(
                &page.pixels()[center..center + 4],
                &[50, 50, 70, 255],
                "BGRA conversion must retain color and opacity"
            );
            assert_eq!(&page.pixels()[0..4], &[255, 255, 255, 255]);
        }
        if id == "long" {
            engine
                .render(key(id, 4999, 2), Priority::Visible, cancel.clone())
                .unwrap();
        }
        engine.close(id.into()).unwrap();
        assert!(
            engine
                .render(key(id, 0, 2), Priority::Visible, cancel.clone())
                .is_err()
        );
        assert_eq!(hash(&path), before, "Original PDF bytes changed");
    }
    assert!(matches!(
        engine.open(
            "password".into(),
            fixture("protected.pdf"),
            None,
            cancel.clone()
        ),
        Err(Failure::PasswordRequired)
    ));
    let before = hash(&fixture("protected.pdf"));
    engine
        .open(
            "password".into(),
            fixture("protected.pdf"),
            Some("fixture-password".into()),
            cancel.clone(),
        )
        .unwrap();
    engine
        .render(key("password", 0, 1), Priority::Visible, cancel.clone())
        .unwrap();
    engine.close("password".into()).unwrap();
    assert_eq!(hash(&fixture("protected.pdf")), before);
    assert!(
        engine
            .open("bad".into(), fixture("damaged.pdf"), None, cancel)
            .is_err()
    );
}

#[test]
fn page_budget_follows_last_live_owner_even_after_eviction_and_worker_close() {
    let bytes = 600 * 800 * 4;
    let engine = PdfWorker::start(
        &root().join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits {
            pages: bytes,
            ..MemoryLimits::default()
        },
    )
    .unwrap();
    let governor = engine.memory_governor();
    engine
        .open(
            "budget".into(),
            fixture("textual.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let first = engine
        .render(
            key("budget", 0, 1),
            Priority::Visible,
            Cancellation::default(),
        )
        .unwrap();
    assert_eq!(governor.snapshot().usage.pages, bytes);
    assert!(matches!(
        engine.render(
            key("budget", 1, 2),
            Priority::Visible,
            Cancellation::default()
        ),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(
        governor.snapshot().usage.pages,
        bytes,
        "Eviction cannot release a consumer's live buffer"
    );
    drop(first);
    let second = engine
        .render(
            key("budget", 1, 2),
            Priority::Visible,
            Cancellation::default(),
        )
        .unwrap();
    engine.close("budget".into()).unwrap();
    drop(engine);
    assert_eq!(
        governor.snapshot().usage.pages,
        bytes,
        "Worker close does not revoke a published page"
    );
    drop(second);
    assert_eq!(governor.snapshot().usage.pages, 0);
}
#[test]
fn two_hot_documents_and_cancelled_consumers_release_cleanly() {
    let engine = engine();
    for id in ["a", "b", "c"] {
        engine
            .open(
                id.into(),
                fixture("textual.pdf"),
                None,
                Cancellation::default(),
            )
            .unwrap();
    }
    assert!(
        engine
            .render(key("a", 0, 0), Priority::Visible, Cancellation::default())
            .is_err()
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        engine.render(key("b", 0, 1), Priority::Visible, cancel),
        Err(Failure::Cancelled)
    ));
    let first = engine
        .render(key("b", 0, 1), Priority::Visible, Cancellation::default())
        .unwrap();
    let second = engine
        .render(key("b", 0, 1), Priority::Visible, Cancellation::default())
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    drop(first);
    drop(second);
    engine.close("b".into()).unwrap();
    engine.close("c".into()).unwrap();
    drop(engine); // Joins the dedicated native worker before returning.
}

#[test]
fn native_tiles_stitch_to_full_render_for_mixed_pages_and_every_rotation() {
    let engine = engine();
    for name in ["textual.pdf", "rotated.pdf", "scanned.pdf"] {
        let path = fixture(name);
        let before = hash(&path);
        let info = engine
            .open("tiles".into(), path.clone(), None, Cancellation::default())
            .unwrap();
        for page in 0..info.page_sizes.len() as u32 {
            for rotation in [0, 90, 180, 270] {
                // Uneven sizes and partitions exercise anisotropic scale, nonzero
                // origins and the final pixel on the bottom/right canvas edges.
                let mut full_key = key("tiles", page, u64::from(rotation));
                full_key.physical_width = 811;
                full_key.physical_height = 613;
                full_key.rotation = rotation;
                let full = engine
                    .render(full_key.clone(), Priority::Visible, Cancellation::default())
                    .unwrap();
                let mut stitched = vec![0; full.pixels().len()];
                for [top, bottom] in [[0, 103], [103, 419], [419, 613]] {
                    for [left, right] in [[0, 127], [127, 517], [517, 811]] {
                        let mut tile_key = full_key.clone();
                        tile_key.region = Some([left, top, right - left, bottom - top]);
                        let tile = engine
                            .render(tile_key, Priority::Visible, Cancellation::default())
                            .unwrap();
                        assert_eq!([tile.width, tile.height], [right - left, bottom - top]);
                        for row in 0..tile.height as usize {
                            let src = row * tile.width as usize * 4;
                            let dst = ((top as usize + row) * 811 + left as usize) * 4;
                            let len = tile.width as usize * 4;
                            stitched[dst..dst + len]
                                .copy_from_slice(&tile.pixels()[src..src + len]);
                        }
                    }
                }
                let differences: Vec<_> = stitched
                    .iter()
                    .zip(full.pixels())
                    .enumerate()
                    .filter_map(|(index, (actual, expected))| {
                        (actual != expected).then_some((index, actual.abs_diff(*expected)))
                    })
                    .collect();
                let total: usize = differences
                    .iter()
                    .map(|(_, difference)| usize::from(*difference))
                    .sum();
                let mean = total as f64 / stitched.len() as f64;
                // PDFium may anti-alias a glyph differently when its outline
                // intersects the destination bitmap boundary. Substantial
                // differences must stay within a boundary-crossing glyph (16
                // device pixels around a partition); elsewhere allow
                // only one-channel rounding. Alpha must always match exactly.
                for &(index, difference) in &differences {
                    let pixel = index / 4;
                    let x = pixel % 811;
                    let y = pixel / 811;
                    let seam = [127_usize, 517].iter().any(|edge| x.abs_diff(*edge) <= 16)
                        || [103_usize, 419].iter().any(|edge| y.abs_diff(*edge) <= 16);
                    assert!(
                        index % 4 != 3 && (difference <= 1 || seam),
                        "non-seam native difference {difference}: {name}, page {page}, rotation {rotation}, ({x},{y})"
                    );
                }
                assert!(
                    mean < 0.01 && differences.len() < stitched.len() / 10000,
                    "native stitching: {name}, page {page}, rotation {rotation}; {} differing channels, mean {mean}, first {:?}",
                    differences.len(),
                    differences.first()
                );
            }
        }
        engine.close("tiles".into()).unwrap();
        assert_eq!(hash(&path), before, "Original bytes changed: {name}");
    }
}

#[test]
fn native_tile_only_allocates_its_buffer_and_rejects_invalid_or_cancelled_regions() {
    let bytes = 73 * 91 * 4;
    let engine = PdfWorker::start(
        &root().join("build/native/pdfium-7881/bin/pdfium.dll"),
        MemoryLimits {
            pages: 0,
            tiles: bytes,
            ..MemoryLimits::default()
        },
    )
    .unwrap();
    let governor = engine.memory_governor();
    let path = fixture("textual.pdf");
    let before = hash(&path);
    engine
        .open(
            "tile-budget".into(),
            path.clone(),
            None,
            Cancellation::default(),
        )
        .unwrap();
    let mut tile_key = key("tile-budget", 0, 1);
    tile_key.physical_width = 16384;
    tile_key.physical_height = 16384;
    tile_key.region = Some([16311, 16293, 73, 91]);
    let tile = engine
        .render(tile_key.clone(), Priority::Visible, Cancellation::default())
        .unwrap();
    assert_eq!(tile.pixels().len(), bytes);
    assert_eq!([tile.width, tile.height], [73, 91]);
    assert_eq!(governor.snapshot().usage.pages, 0);
    assert_eq!(governor.snapshot().usage.tiles, bytes);
    let mut another = tile_key.clone();
    another.region = Some([0, 0, 73, 91]);
    another.generation += 1;
    assert!(matches!(
        engine.render(another, Priority::Visible, Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(
        governor.snapshot().usage.tiles,
        bytes,
        "Tile cache eviction cannot release a live consumer's buffer"
    );
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(matches!(
        engine.render(tile_key.clone(), Priority::Visible, cancelled),
        Err(Failure::Cancelled)
    ));
    for region in [[16312, 16293, 73, 91], [u32::MAX, 0, 73, 91], [0, 0, 0, 91]] {
        tile_key.region = Some(region);
        assert!(matches!(
            engine.render(tile_key.clone(), Priority::Visible, Cancellation::default()),
            Err(Failure::Invalid(_))
        ));
        assert_eq!(governor.snapshot().usage.tiles, bytes);
    }
    engine.close("tile-budget".into()).unwrap();
    assert_eq!(governor.snapshot().usage.tiles, bytes);
    drop(tile);
    assert_eq!(governor.snapshot().usage.tiles, 0);
    assert_eq!(hash(&path), before);
}

#[test]
fn admitted_async_tickets_complete_once_and_shutdown_joins_with_replies_undrained() {
    use yamantaka_domain::pdf_ticket::AsyncPdfEngine;
    let engine = engine();
    engine
        .try_open(
            "async".into(),
            fixture("textual.pdf"),
            None,
            Cancellation::default(),
        )
        .unwrap()
        .wait()
        .unwrap();
    let tickets: Vec<_> = (0..128)
        .map(|_| {
            engine
                .try_render(
                    key("async", 0, 1),
                    Priority::Visible,
                    Cancellation::default(),
                )
                .unwrap()
        })
        .collect();
    let identities: std::collections::HashSet<_> =
        tickets.iter().map(|ticket| ticket.consumer_id).collect();
    assert_eq!(identities.len(), 128);
    assert!(
        engine
            .try_render(
                key("async", 0, 1),
                Priority::Visible,
                Cancellation::default()
            )
            .is_err()
    );
    let close = engine.try_close("async".into()).unwrap();
    let governor = engine.memory_governor();
    let (send, wait) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        drop(engine);
        send.send(()).unwrap();
    });
    wait.recv_timeout(std::time::Duration::from_secs(5))
        .expect("PDF shutdown must join even with 128 undrained replies");
    for ticket in tickets {
        match ticket
            .try_recv()
            .expect("every admitted ticket has a terminal result after join")
        {
            Ok(page) => assert_eq!(page.pixels().len(), 600 * 800 * 4),
            Err(Failure::Cancelled) => {}
            Err(error) => panic!("unexpected terminal error: {error}"),
        }
        assert!(
            matches!(
                ticket.try_recv(),
                Err(crossbeam_channel::TryRecvError::Disconnected)
            ),
            "ticket completed twice"
        );
    }
    let _ = close.wait();
    assert_eq!(governor.snapshot().usage.pages, 0);
}

#[test]
fn concurrent_long_document_inspections_finish_before_hot_handle_eviction() {
    use yamantaka_domain::pdf_ticket::AsyncPdfEngine;
    let engine = engine();
    let tickets: Vec<_> = (0..3)
        .map(|id| {
            engine
                .try_open(
                    format!("long-{id}"),
                    fixture("5000-pages.pdf"),
                    None,
                    Cancellation::default(),
                )
                .unwrap()
        })
        .collect();
    for ticket in tickets {
        assert_eq!(ticket.wait().unwrap().page_sizes.len(), 5000);
    }
}
