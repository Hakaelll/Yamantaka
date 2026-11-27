use super::*;
use std::{path::PathBuf, sync::mpsc};
use yamantaka_domain::{
    Failure, Result,
    pdf_ticket::PdfTicket,
    study_library::{PdfMark, PdfMarks},
};
use yamantaka_pdf::PdfWorker;

#[derive(Default)]
pub(super) struct State {
    engine: Option<PdfWorker>,
    settings: Option<Settings>,
    active: Option<Job>,
    status: Option<Result<usize>>,
}
impl State {
    pub(super) fn open(&self) -> bool {
        self.settings.is_some()
    }
}

struct Settings {
    document: String,
    fingerprint: String,
    reference: bool,
    mode: Mode,
    first: u32,
    last: u32,
    count: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Recognize,
    Export,
}

struct Job {
    cancel: Cancellation,
    result: Work,
}

enum Work {
    Marks,
    Export(PdfTicket<()>),
    Recognize(mpsc::Receiver<Result<usize>>),
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl YamantakaApp {
    pub(crate) fn install_study_engine(&mut self, engine: PdfWorker) {
        self.study_document.engine = Some(engine);
    }

    pub(super) fn study_document_actions(&mut self, ui: &mut egui::Ui, reference: bool) {
        ui.separator();
        if ui
            .button(self.tr("Reconocer texto · OCR", "Recognize text · OCR"))
            .clicked()
        {
            self.begin_study_document(reference, Mode::Recognize);
            ui.close();
        }
        if ui
            .button(self.tr("Copia PDF con marcas", "Marked PDF copy"))
            .clicked()
        {
            self.begin_study_document(reference, Mode::Export);
            ui.close();
        }
    }

    fn begin_study_document(&mut self, reference: bool, mode: Mode) {
        if self.study_document.active.is_some() {
            return;
        }
        let pane = self.study_pane(reference);
        let Some((document, info)) = &pane.active else {
            return;
        };
        if pane.viewport.source_changed {
            return;
        }
        self.study_document.settings = Some(Settings {
            document: document.id.clone(),
            fingerprint: document.fingerprint.clone(),
            reference,
            mode,
            first: pane.page + 1,
            last: pane.page + 1,
            count: info.pages.len() as u32,
        });
        self.study_document.status = None;
    }

    fn start_study_document(
        &mut self,
        ctx: &egui::Context,
        destination: Option<PathBuf>,
    ) -> Result<()> {
        let settings = self
            .study_document
            .settings
            .as_ref()
            .ok_or(Failure::Cancelled)?;
        let Some(engine) = self.study_document.engine.clone() else {
            return Err(Failure::Unsupported(
                "Document study tools are unavailable".into(),
            ));
        };
        let pages = settings.first - 1..settings.last;
        let cancel = Cancellation::default();
        let work = match settings.mode {
            Mode::Export => {
                let Some(destination) = destination else {
                    return Ok(());
                };
                self.runtime.submit(Action::StudyPdfMarks(
                    Box::new(PdfMarks {
                        document: settings.document.clone(),
                        fingerprint: settings.fingerprint.clone(),
                        pages,
                        destination,
                    }),
                    cancel.clone(),
                ))?;
                Work::Marks
            }
            Mode::Recognize => {
                let helper = ocr_helper().ok_or_else(|| {
                    Failure::Unsupported(
                        self.tr(
                            "Falta el complemento local en ocr/yamantaka-ocr.exe.",
                            "The local add-on is missing at ocr/yamantaka-ocr.exe.",
                        )
                        .into(),
                    )
                })?;
                let pane = self.study_pane(settings.reference);
                let Some((_, info)) = &pane.active else {
                    return Err(Failure::Cancelled);
                };
                let pages = pages
                    .map(|page| (page, info.pages.get(page as usize).unwrap().size()))
                    .collect();
                let id = settings.document.clone();
                let worker_cancel = cancel.clone();
                let repaint = ctx.clone();
                let (send, receive) = mpsc::channel();
                std::thread::Builder::new()
                    .name("yamantaka-ocr-job".into())
                    .spawn(move || {
                        let _ =
                            send.send(engine.recognize_pages(id, pages, &helper, &worker_cancel));
                        repaint.request_repaint();
                    })
                    .map_err(|_| Failure::Pdf("OCR worker could not start".into()))?;
                Work::Recognize(receive)
            }
        };
        self.study_document.active = Some(Job {
            cancel,
            result: work,
        });
        self.study_document.status = None;
        Ok(())
    }

    pub(super) fn accept_study_pdf_marks(
        &mut self,
        request: PdfMarks,
        result: Result<Vec<PdfMark>>,
        cancel: Cancellation,
    ) {
        let state = &mut self.study_document;
        let Some(job) = &mut state.active else {
            return;
        };
        if !matches!(job.result, Work::Marks) || !job.cancel.same_request(&cancel) {
            return;
        }
        let result = cancel.check().and(result).and_then(|marks| {
            state
                .engine
                .as_ref()
                .ok_or(Failure::Cancelled)?
                .export_marked_pdf(request, marks, cancel)
        });
        match result {
            Ok(ticket) => job.result = Work::Export(ticket),
            Err(error) => {
                state.active = None;
                state.status = Some(Err(error));
            }
        }
    }

    pub(super) fn study_document_window(&mut self, ctx: &egui::Context) {
        self.poll_study_document(ctx);
        if self.study_document.settings.is_some()
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if let Some(job) = &self.study_document.active {
                job.cancel.cancel();
            } else {
                self.study_document.settings = None;
            }
        }
        let Some(settings) = self.study_document.settings.as_ref() else {
            return;
        };
        let english = self.english;
        let tr = |es, en| if english { en } else { es };
        let viewport = ctx.content_rect();
        let mode = settings.mode;
        let reference = settings.reference;
        let source = if reference && self.reference.is_none() {
            None
        } else {
            self.study_pane(reference)
                .active
                .as_ref()
                .filter(|(doc, _)| {
                    doc.id == settings.document
                        && doc.fingerprint == settings.fingerprint
                        && !self.study_pane(reference).viewport.source_changed
                })
        };
        let present = source.is_some();
        let current_page = present.then(|| self.study_pane(reference).page + 1);
        let ready = mode != Mode::Recognize || ocr_helper().is_some();
        let mut running = self.study_document.active.as_ref();
        let mut open = true;
        let mut preview = false;
        let mut start = false;
        let title = if mode == Mode::Recognize {
            tr("OCR local", "Local OCR")
        } else {
            tr("Copia PDF con marcas", "Marked PDF copy")
        };
        let settings = self.study_document.settings.as_mut().unwrap();
        egui::Window::new(title).id(egui::Id::new("study-document-action")).open(&mut open).collapsible(false)
            .default_width((viewport.width() - 30.0).clamp(170.0, 360.0))
            .max_width((viewport.width() - 22.0).max(170.0))
            .show(ctx, |ui| {
                if let Some(job) = running.take() {
                    ui.horizontal_wrapped(|ui| {
                        ui.spinner();
                        ui.label(if mode == Mode::Recognize { tr("Reconociendo…", "Recognizing…") } else { tr("Preparando PDF…", "Preparing PDF…") });
                        if ui.add_enabled(!job.cancel.is_cancelled(), egui::Button::new(tr("Cancelar", "Cancel"))).clicked() { job.cancel.cancel(); }
                    });
                } else {
                    if mode == Mode::Recognize {
                        ui.label(tr("Reconoce hasta 24 páginas sin capa de texto en una sola operación.", "Recognize up to 24 pages without a text layer at once."));
                        if !ready { ui.label(tr("Instala el complemento en ocr/yamantaka-ocr.exe junto a la aplicación.", "Install the add-on at ocr/yamantaka-ocr.exe alongside the app.")); }
                    } else {
                        ui.label(tr("La copia incluye los resaltados, subrayados y dibujos adjuntos del rango.", "The copy includes attached highlights, underlines, and drawings in the page range."));
                    }
                    ui.add_enabled_ui(present, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let label = ui.label(tr("Desde", "From"));
                            ui.add(egui::DragValue::new(&mut settings.first).range(1..=settings.count)).labelled_by(label.id);
                            let label = ui.label(tr("Hasta", "To"));
                            ui.add(egui::DragValue::new(&mut settings.last).range(1..=settings.count)).labelled_by(label.id);
                        });
                        ui.horizontal_wrapped(|ui| {
                            if ui.button(tr("Página actual", "Current page")).clicked() {
                                settings.first = current_page.unwrap_or(1);
                                settings.last = settings.first;
                            }
                            if mode == Mode::Export && ui.button(tr("Todas", "All pages")).clicked() {
                                settings.first = 1; settings.last = settings.count;
                            }
                        });
                        let valid = settings.first <= settings.last && (mode == Mode::Export || settings.last - settings.first < 24);
                        ui.horizontal_wrapped(|ui| {
                            if mode == Mode::Export {
                                preview = ui.add_enabled(valid, egui::Button::new(tr("Vista previa", "Preview"))).clicked();
                            }
                            start = ui.add_enabled(ready && valid, egui::Button::new(if mode == Mode::Export {
                                tr("Guardar copia", "Save copy")
                            } else { tr("Reconocer", "Recognize") })).clicked();
                        });
                    });
                }
                if let Some(result) = &self.study_document.status {
                    match result {
                        Ok(count) if mode == Mode::Recognize => {
                            ui.label(if english { format!("Recognized {count} pages. Use Select or Find on the PDF.") }
                                else { format!("{count} páginas reconocidas. Usa Seleccionar o Buscar en el PDF.") });
                        }
                        Ok(_) => { ui.label(tr("Copia PDF guardada.", "PDF copy saved.")); }
                        Err(error) => { ui.label(error.to_string()); }
                    }
                }
            });
        let first = settings.first;
        if !open && self.study_document.active.is_none() {
            self.study_document.settings = None;
        }
        if preview {
            let pane = self.study_pane_mut(reference);
            pane.study.hidden = false;
            if let Some(nav) = &mut pane.navigation {
                nav.go_to_page(first - 1);
            }
            if reference {
                self.reference
                    .as_mut()
                    .unwrap()
                    .reference_navigation_changed();
            } else {
                self.apply_navigation();
            }
        }
        if start {
            let destination = if mode == Mode::Export {
                match yamantaka_windows::annotation_export::pick_annotation_export(title, "pdf") {
                    Ok(Some(path)) => Some(path),
                    Ok(None) => return,
                    Err(error) => {
                        self.study_document.status = Some(Err(error));
                        return;
                    }
                }
            } else {
                None
            };
            if let Err(error) = self.start_study_document(ctx, destination) {
                self.study_document.status = Some(Err(error));
            }
        }
    }

    fn poll_study_document(&mut self, ctx: &egui::Context) {
        let Some(job) = self.study_document.active.take() else {
            return;
        };
        let result = match &job.result {
            Work::Marks => None,
            Work::Export(ticket) if ticket.is_ready() => Some(
                ticket
                    .try_recv()
                    .unwrap_or_else(|_| Err(Failure::Pdf("PDF export worker stopped".into())))
                    .map(|()| 0),
            ),
            Work::Export(_) => None,
            Work::Recognize(receiver) => match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(Failure::Pdf("OCR worker stopped".into())))
                }
            },
        };
        let Some(result) = result else {
            self.study_document.active = Some(job);
            ctx.request_repaint_after(Duration::from_millis(90));
            return;
        };
        if matches!(job.result, Work::Recognize(_))
            && result.is_ok()
            && let Some(settings) = &self.study_document.settings
        {
            let id = settings.document.clone();
            let range = settings.first - 1..settings.last;
            reset_text_geometry(&mut self.reader, &id, range.clone());
            if let Some(reference) = &mut self.reference {
                reset_text_geometry(&mut reference.pane, &id, range);
            }
        }
        self.study_document.status = Some(result);
    }
}

fn ocr_helper() -> Option<PathBuf> {
    let path = std::env::current_exe()
        .ok()?
        .parent()?
        .join("ocr/yamantaka-ocr.exe");
    path.is_file().then_some(path)
}

fn reset_text_geometry(pane: &mut reader_pane::ReaderPane, id: &str, pages: std::ops::Range<u32>) {
    if pane.active.as_ref().is_some_and(|(doc, _)| doc.id == id) {
        pane.selection.dismiss_passage();
        for page in pages {
            pane.selection.retry_page(page);
        }
        pane.text.clear();
        pane.find.stop();
    }
}
