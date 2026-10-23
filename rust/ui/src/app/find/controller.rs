use super::*;

impl FindState {
    fn submit_find(
        &mut self,
        runtime: &Runtime,
        request: FindRequest,
        edge: SelectionEdge,
        wrap: bool,
    ) {
        self.stop();
        let token = request.cancellation.clone();
        let progress = request.progress.clone();
        self.document = Some(request.document_id.clone());
        self.phase = match runtime.submit(Action::FindDocument(request)) {
            Ok(()) => FindPhase::Running {
                token,
                progress,
                edge,
                wrap,
            },
            Err(error) => FindPhase::Failed(error.to_string()),
        };
    }
    pub(in crate::app) fn start(&mut self, runtime: &Runtime, document: &Document, info: &PdfInfo) {
        let request =
            PdfFindQuery::new(&self.editor, self.match_case, self.whole_words).and_then(|query| {
                self.query = Some(query.clone());
                FindRequest::new(
                    document.id.clone(),
                    query,
                    info.pages.len() as u32,
                    FindCursor::default(),
                    Cancellation::default(),
                )
            });
        match request {
            Ok(request) => self.submit_find(runtime, request, SelectionEdge::First, false),
            Err(error) => {
                self.stop();
                self.phase = FindPhase::Failed(error.to_string());
            }
        }
    }
    pub(in crate::app) fn choose(&mut self, index: usize) -> Option<FindTarget> {
        let FindPhase::Ready { batch, selected } = &mut self.phase else {
            return None;
        };
        let (page, item) = batch.matches().nth(index)?;
        *selected = Some(index);
        Some(FindTarget {
            page,
            position: item.position(page),
        })
    }

    fn wrap_find(
        &mut self,
        runtime: &Runtime,
        document: &Document,
        info: &PdfInfo,
        edge: SelectionEdge,
    ) {
        let Some(query) = self.query.clone() else {
            return;
        };
        let request = match edge {
            SelectionEdge::First => FindRequest::new(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                FindCursor::default(),
                Cancellation::default(),
            ),
            SelectionEdge::Last => FindRequest::previous(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                None,
                Cancellation::default(),
            ),
        };
        match request {
            Ok(request) => self.submit_find(runtime, request, edge, false),
            Err(error) => {
                self.stop();
                self.phase = FindPhase::Failed(error.to_string());
            }
        }
    }
    pub(in crate::app) fn step(
        &mut self,
        runtime: &Runtime,
        document: &Document,
        info: &PdfInfo,
        forward: bool,
    ) -> Option<FindTarget> {
        let FindPhase::Ready {
            batch,
            selected: Some(selected),
        } = &self.phase
        else {
            return None;
        };
        let count = batch.matches().count();
        let inside = if forward {
            (*selected + 1 < count).then_some(*selected + 1)
        } else {
            selected.checked_sub(1)
        };
        if let Some(index) = inside {
            return self.choose(index);
        }
        let Some(query) = self.query.clone() else {
            return None;
        };
        let wrap = if forward {
            batch.next.is_some()
        } else {
            batch
                .start
                .is_some_and(|start| start != FindCursor::default())
        };
        let cancel = Cancellation::default();
        let request = if forward {
            FindRequest::new(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                batch.next.unwrap_or_default(),
                cancel,
            )
        } else {
            let boundary = batch.start.filter(|start| *start != FindCursor::default());
            FindRequest::previous(
                document.id.clone(),
                query,
                info.pages.len() as u32,
                boundary,
                cancel,
            )
        };
        match request {
            Ok(request) => self.submit_find(
                runtime,
                request,
                if forward {
                    SelectionEdge::First
                } else {
                    SelectionEdge::Last
                },
                wrap,
            ),
            Err(error) => {
                self.stop();
                self.phase = FindPhase::Failed(error.to_string());
            }
        }
        None
    }
    pub(in crate::app) fn accept_result(
        &mut self,
        runtime: &Runtime,
        document: &Document,
        info: &PdfInfo,
        cancel: &Cancellation,
        result: Result<FindBatch, String>,
    ) -> Option<FindTarget> {
        let wrap_edge = match &self.phase {
            FindPhase::Running {
                edge, wrap: true, ..
            } => Some(*edge),
            _ => None,
        };
        if !self.accept(&document.id, cancel, result) {
            return None;
        }
        if let Some(edge) = wrap_edge
            && matches!(&self.phase, FindPhase::Ready { batch, .. } if batch.matches().next().is_none())
        {
            self.wrap_find(runtime, document, info, edge);
            return None;
        }
        let index = match &self.phase {
            FindPhase::Ready { selected, .. } => *selected,
            _ => None,
        };
        index.and_then(|index| self.choose(index))
    }

    pub(in crate::app) fn paint(
        &self,
        ui: &egui::Ui,
        page: u32,
        paper: egui::Rect,
        rotation: yamantaka_domain::reader_preferences::PageRotation,
        p: Palette,
    ) {
        let FindPhase::Ready { batch, selected } = &self.phase else {
            return;
        };
        for (index, (match_page, item)) in batch.matches().enumerate() {
            if match_page != page {
                continue;
            }
            for bounds in &item.rectangles {
                let rect = reader_viewport::page_region(paper, rotation, *bounds);
                let active = *selected == Some(index);
                ui.painter().rect_filled(
                    rect,
                    1,
                    p.accent.gamma_multiply(if active { 0.30 } else { 0.14 }),
                );
                if active {
                    ui.painter().rect_stroke(
                        rect,
                        1,
                        egui::Stroke::new(1.0, p.accent),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }
}
