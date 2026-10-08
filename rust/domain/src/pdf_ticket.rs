//! Nonblocking, one-consumer PDF replies. Native handles never leave their owner.
use crate::{Cancellation, PdfEngine, PdfInfo, Priority, RenderKey, RenderedPage, Result};
use crossbeam_channel::{Receiver, Select, Sender, TryRecvError, bounded};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct PdfTicket<T> {
    receiver: Receiver<Result<T>>,
    pub consumer_id: u64,
    pub epoch: u64,
    pub cancellation: Cancellation,
    lifetime: Mutex<Option<Arc<dyn Send + Sync>>>,
}
impl<T> PdfTicket<T> {
    pub fn channel(
        consumer_id: u64,
        epoch: u64,
        cancellation: Cancellation,
    ) -> (Sender<Result<T>>, Self) {
        let (sender, receiver) = bounded(1);
        (
            sender,
            Self {
                receiver,
                consumer_id,
                epoch,
                cancellation,
                lifetime: Mutex::new(None),
            },
        )
    }
    pub fn retain(mut self, lifetime: Arc<dyn Send + Sync>) -> Self {
        *self.lifetime.get_mut().unwrap() = Some(lifetime);
        self
    }
    pub fn try_recv(&self) -> std::result::Result<Result<T>, TryRecvError> {
        let result = self.receiver.try_recv();
        if result.is_ok() {
            self.lifetime.lock().unwrap().take();
        }
        result
    }
    pub fn wait(self) -> Result<T> {
        self.receiver
            .recv()
            .unwrap_or_else(|_| Err(crate::Failure::Pdf("PDF worker stopped".into())))
    }
    /// Register readiness in a coordinator's blocking select; this never waits.
    pub fn readiness(&self) -> &Receiver<Result<T>> {
        &self.receiver
    }
    /// Disconnection is ready as well: coordinators must publish worker failure,
    /// rather than spin forever on an empty but closed response channel.
    pub fn is_ready(&self) -> bool {
        let mut select = Select::new();
        select.recv(&self.receiver);
        select.try_ready().is_ok()
    }
}
pub trait AsyncPdfEngine: PdfEngine {
    fn try_open(
        &self,
        id: String,
        path: PathBuf,
        password: Option<String>,
        cancel: Cancellation,
    ) -> Result<PdfTicket<PdfInfo>>;
    fn try_render(
        &self,
        key: RenderKey,
        priority: Priority,
        cancel: Cancellation,
    ) -> Result<PdfTicket<Arc<RenderedPage>>>;
    fn try_text(&self, id: String, page: u32, cancel: Cancellation) -> Result<PdfTicket<String>>;
    fn try_close(&self, id: String) -> Result<PdfTicket<()>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnected_empty_ticket_is_ready_and_reports_failure() {
        let (sender, ticket) = PdfTicket::<()>::channel(1, 1, Cancellation::default());
        assert!(!ticket.is_ready());
        drop(sender);
        assert!(ticket.is_ready());
        assert!(ticket.wait().is_err());
    }
}
