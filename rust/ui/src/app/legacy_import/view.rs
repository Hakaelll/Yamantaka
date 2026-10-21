use super::*;
impl YamantakaApp {
    pub(in crate::app) fn legacy_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.legacy_import.open {
            return;
        }
        let mut state = std::mem::take(&mut self.legacy_import);
        let p = Palette::new(self.dark);
        let compact = ctx.content_rect().height() < 420.0;
        let mut close = false;
        let modal=egui::Modal::new(egui::Id::new("legacy-import-review")).frame(egui::Frame::new().fill(p.surface).stroke(egui::Stroke::new(1.0,p.line)).corner_radius(12).inner_margin(if compact {12}else{24})).show(ctx,|ui| {
            ui.set_width((ctx.content_rect().width()-56.0).clamp(180.0,560.0));
            ui.spacing_mut().item_spacing.y=if compact {6.0}else{12.0};
            theme::display(ui,self.tr("Importar desde Flutter","Import from Flutter"),if compact {20.0}else{26.0});
            let body_height=(ctx.content_rect().height()-if compact {164.0}else{180.0}).clamp(40.0,480.0);
            egui::ScrollArea::vertical().id_salt("legacy-import-body").max_height(body_height).show(ui,|ui| {
                if let Some(job)=&state.job {
                    let text=match state.pending {
                        Some(Pending::Review)=>self.tr("Comprobando el origen…","Checking the source…"),
                        Some(Pending::Activate|Pending::Original)=>self.tr("Guardando la selección…","Saving the selection…"),
                        _=>match job.progress() {Some(MigrationProgress::Copying {..})=>self.tr("Copiando la biblioteca…","Copying the library…"),Some(MigrationProgress::Installing)=>self.tr("Finalizando la copia…","Finishing the copy…"),_=>self.tr("Validando los datos…","Validating the data…")},
                    };
                    ui.label(text);
                    if let Some(MigrationProgress::Copying {copied_pages,total_pages})=job.progress() {ui.label(format!("{copied_pages} / {total_pages} {}",self.tr("páginas de base de datos","database pages")));}
                }
                if let Some(error)=&state.error {
                    let message=match error {
                        Failure::LegacyRunning=>self.tr("Cierra Flutter y vuelve a revisar el origen.","Close Flutter and review the source again."),
                        Failure::MigrationSourceChanged=>self.tr("El origen ha cambiado. Vuelve a revisarlo antes de copiar.","The source changed. Review it again before copying."),
                        Failure::Cancelled=>self.tr("Operación cancelada. La biblioteca actual se conserva.","Operation cancelled. The current library is kept."),
                        Failure::DestinationOccupied=>self.tr("Ese destino ya contiene datos. Vuelve a revisar para crear otra copia.","That destination already contains data. Review again to create another copy."),
                        _=>self.tr("No se pudo completar la operación. Comprueba los archivos y el acceso y vuelve a intentarlo.","The operation could not finish. Check the files and access, then retry."),
                    };
                    ui.label(RichText::new(message).color(p.accent));
                }
                if state.original_next {
                    ui.label(self.tr("La biblioteca Rust original se usará al volver a abrir Yamantaka. La copia importada se conserva.","The original Rust library will be used when you reopen Yamantaka. The imported copy is kept."));
                } else if state.selected_next {
                    ui.label(RichText::new(self.tr("Lista para el próximo arranque","Ready for next launch")).strong());
                    ui.label(self.tr("Cierra y vuelve a abrir Yamantaka para usar esta copia. Tu biblioteca actual se conserva.","Close and reopen Yamantaka to use this copy. Your current library is kept."));
                } else if let Some(receipt)=&state.receipt {
                    ui.label(RichText::new(self.tr("Copia creada y comprobada","Copy created and checked")).strong());
                    self.import_counts(ui,&receipt.imported_rows,p);
                    ui.label(self.tr("La copia no se sincroniza con Flutter. Puedes elegirla para el próximo arranque.","The copy does not sync with Flutter. You can choose it for the next launch."));
                } else if let Some(review)=&state.review {
                    ui.label(RichText::new(self.tr("Revisa lo que se copiará","Review what will be copied")).strong());
                    let source=if self.demo && self.capture.is_some() {"Synthetic Flutter library".into()}else{review.source().database.to_string_lossy().into_owned()};
                    ui.label(RichText::new(source).size(12.0).color(p.secondary));
                    self.import_counts(ui,review.imported_rows(),p);
                    ui.label(format!("{} {}",review.preferences_count(),self.tr("ajustes conservados","settings preserved")));
                    ui.label(self.tr("Se conservarán identificadores, relaciones, progreso y decisiones manuales. Los PDF permanecen en sus ubicaciones.","Identifiers, relationships, progress and manual choices will be kept. PDFs stay in their existing locations."));
                } else {
                    if state.error.is_none() {ui.label(self.tr("Elige la base de datos y las preferencias de Flutter. Cierra Flutter antes de revisar.","Choose the Flutter database and preferences. Close Flutter before reviewing."));}
                    let mut changed=false;
                    ui.add_enabled_ui(!state.busy(),|ui| {
                        changed|=self.import_file_row(ui,&mut state,true,p);
                        changed|=self.import_file_row(ui,&mut state,false,p);
                    });
                    if changed {state.reset_review();}
                    ui.label(RichText::new(self.tr("En modo portátil, la base está en data; las preferencias siguen en Windows.","In portable mode, the database is in data; preferences still live in Windows.")).color(p.secondary).size(12.0));
                }
            });
            ui.horizontal_wrapped(|ui| {
                let busy=state.busy();
                let cancel=theme::navigation(ui,if busy {self.tr("Cancelar","Cancel")}else{self.tr("Cerrar","Close")},false,p);
                if state.focus_cancel {cancel.request_focus();state.focus_cancel=false;}
                if cancel.clicked() {if busy {state.cancel();}else{close=true;}}
                if !busy && state.selected_next && theme::navigation(ui,self.tr("Usar original","Use original"),false,p).clicked() && let Some(root)=state.root.clone() {
                    state.begin(Command::Original(root),Pending::Original,ctx);
                }
                if !busy && !state.selected_next && !state.original_next {
                    let label=if state.receipt.is_some() {self.tr("Usar al volver a abrir","Use next launch")}else if state.review.is_some() {self.tr("Crear copia independiente","Create independent copy")}else{self.tr("Revisar origen","Review source")};
                    if theme::control(ui,label,None,false,true,p).clicked() {
                        if state.receipt.is_some() {state.activate(ctx);} else if state.review.is_some() {state.copy(ctx);} else {state.inspect(ctx);}
                    }
                    if state.review.is_some() && theme::navigation(ui,self.tr("Cambiar origen","Change source"),false,p).clicked() {state.reset_review();}
                }
            });
        });
        if close || modal.should_close() {
            state.open = false;
            state.cancel();
        }
        self.legacy_import = state;
    }
    fn import_file_row(
        &self,
        ui: &mut egui::Ui,
        state: &mut ImportState,
        database: bool,
        p: Palette,
    ) -> bool {
        let label = if database {
            self.tr("Base de datos", "Database")
        } else {
            self.tr("Preferencias", "Preferences")
        };
        ui.label(RichText::new(label).strong());
        let path = state.source.as_ref().map(|source| {
            if database {
                &source.database
            } else {
                &source.preferences
            }
        });
        let caption = path
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.tr("Sin seleccionar", "Not selected").into());
        ui.label(caption);
        if let Some(path) = path {
            let location = if self.demo && self.capture.is_some() {
                "Synthetic Flutter library".into()
            } else {
                path.to_string_lossy().into_owned()
            };
            ui.label(RichText::new(location).size(12.0).color(p.secondary));
        }

        if theme::navigation(
            ui,
            if database {
                self.tr("Elegir base de datos", "Choose database")
            } else {
                self.tr("Elegir preferencias", "Choose preferences")
            },
            false,
            p,
        )
        .clicked()
        {
            let selected = if database {
                yamantaka_windows::pick_flutter_database()
            } else {
                yamantaka_windows::pick_flutter_preferences()
            };
            if let (Some(selected), Some(source)) = (selected, &mut state.source) {
                if database {
                    source.database = selected;
                } else {
                    source.preferences = selected;
                }
                return true;
            }
        }
        false
    }
    fn import_counts(
        &self,
        ui: &mut egui::Ui,
        rows: &std::collections::BTreeMap<String, u64>,
        p: Palette,
    ) {
        for (table, es, en) in [
            ("documents", "Libros", "Books"),
            ("annotations", "Anotaciones", "Annotations"),
            ("bookmarks", "Marcadores", "Bookmarks"),
            ("collections", "Colecciones", "Collections"),
            ("tags", "Etiquetas", "Tags"),
        ] {
            ui.label(format!(
                "{}: {}",
                self.tr(es, en),
                rows.get(table).copied().unwrap_or(0)
            ));
        }
        ui.label(
            RichText::new(self.tr(
                "También se conservan sesiones, espacios y portadas elegidas.",
                "Reading sessions, workspaces and chosen covers are also kept.",
            ))
            .color(p.secondary)
            .size(12.0),
        );
    }
}
