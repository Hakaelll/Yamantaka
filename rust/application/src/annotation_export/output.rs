use std::io::{self, Write};
use yamantaka_domain::{
    Cancellation, Failure, Result,
    memory::{MemoryCategory, MemoryGovernor, TrackedBytesBuilder},
};

const BUFFER_BYTES: usize = 64 * 1024;
pub(super) struct ExportOutput<W: Write> {
    writer: W,
    buffer: TrackedBytesBuilder,
    used: usize,
    cancel: Cancellation,
    failed: bool,
}
impl<W: Write> ExportOutput<W> {
    pub(super) fn new(writer: W, governor: &MemoryGovernor, cancel: Cancellation) -> Result<Self> {
        cancel.check()?;
        let buffer = TrackedBytesBuilder::try_zeroed(governor, MemoryCategory::Other, BUFFER_BYTES)
            .map_err(|_| Failure::MemoryBudget)?;
        Ok(Self {
            writer,
            buffer,
            used: 0,
            cancel,
            failed: false,
        })
    }
    pub(super) fn into_inner(self) -> W {
        self.writer
    }
    fn check(&mut self) -> io::Result<()> {
        if self.failed || self.cancel.is_cancelled() {
            self.failed = true;
            Err(io::Error::other("Annotation export output stopped"))
        } else {
            Ok(())
        }
    }
    fn drain(&mut self) -> io::Result<()> {
        let mut offset = 0;
        while offset < self.used {
            self.check()?;
            match self
                .writer
                .write(&self.buffer.as_slice()[offset..self.used])
            {
                Ok(0) => {
                    self.failed = true;
                    return Err(io::ErrorKind::WriteZero.into());
                }
                Ok(count) => offset += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    self.failed = true;
                    return Err(error);
                }
            }
        }
        self.used = 0;
        Ok(())
    }
}
impl<W: Write> Write for ExportOutput<W> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        self.check()?;
        if input.is_empty() {
            return Ok(0);
        }
        if self.used == BUFFER_BYTES {
            self.drain()?;
        }
        self.check()?;
        let count = input.len().min(BUFFER_BYTES - self.used);
        self.buffer.as_mut_slice()[self.used..self.used + count].copy_from_slice(&input[..count]);
        self.used += count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.check()?;
        self.drain()?;
        self.check()?;
        if let Err(error) = self.writer.flush() {
            self.failed = true;
            return Err(error);
        }
        self.check()
    }
}
// Pending bytes are deliberately discarded on drop; only explicit successful finalization writes them.

#[cfg(test)]
mod tests;
