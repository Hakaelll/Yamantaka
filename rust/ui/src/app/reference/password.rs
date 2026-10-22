use super::*;
use yamantaka_domain::PdfPassword;

impl YamantakaApp {
    pub(in crate::app) fn retry_reference_password(
        &mut self,
        document: Document,
        value: PdfPassword,
    ) {
        let Some(reference) = &mut self.reference else {
            return;
        };
        if !matches!(&reference.admission, Admission::Password {document: id} if id == &document.id)
        {
            return;
        }
        let cancel = Cancellation::default();
        match self
            .runtime
            .submit(Action::Open(document.clone(), Some(value), cancel.clone()))
        {
            Ok(()) => {
                if self.demo {
                    self.reference_password_attempts =
                        self.reference_password_attempts.saturating_add(1);
                }
                reference.password_attempted = true;
                reference.admission = Admission::Opening {
                    document: document.id,
                    cancel,
                };
            }
            Err(error) => {
                reference.admission = Admission::Ready;
                reference.error = Some(error.to_string());
            }
        }
    }

    pub(in crate::app) fn cancel_reference_password(&mut self, document: &str) {
        if let Some(reference) = &mut self.reference
            && matches!(&reference.admission, Admission::Password {document: id} if id == document)
        {
            reference.admission = Admission::Ready;
            reference.password_attempted = false;
        }
    }
}
