//! A one-shot credential handoff, independent of persistent import intents.
use crossbeam_channel::{Receiver, Sender, bounded};
use std::path::{Path, PathBuf};
use yamantaka_domain::{Cancellation, Failure, PdfPassword, Result};

pub struct Challenge {
    path: PathBuf,
    rejected: bool,
    cancel: Cancellation,
    reply: Option<Sender<PdfPassword>>,
    wake: Sender<()>,
}
impl Challenge {
    pub(crate) fn channel(
        path: PathBuf,
        rejected: bool,
        cancel: Cancellation,
        wake: Sender<()>,
    ) -> (Self, Receiver<PdfPassword>) {
        let (reply, receive) = bounded(1);
        (
            Self {
                path,
                rejected,
                cancel,
                reply: Some(reply),
                wake,
            },
            receive,
        )
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn rejected(&self) -> bool {
        self.rejected
    }
    pub fn cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
    pub fn answer(mut self, password: PdfPassword) -> Result<()> {
        self.cancel.check()?;
        self.reply
            .take()
            .unwrap()
            .try_send(password)
            .map_err(|_| Failure::Cancelled)?;
        let _ = self.wake.try_send(());
        Ok(())
    }
}
impl Drop for Challenge {
    fn drop(&mut self) {
        if self.reply.is_some() {
            self.cancel.cancel();
            let _ = self.wake.try_send(());
        }
    }
}
