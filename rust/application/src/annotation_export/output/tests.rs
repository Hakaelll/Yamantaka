use super::*;
use yamantaka_domain::memory::GlobalMemoryLimits;

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    calls: usize,
    max_write: usize,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        self.max_write = self.max_write.max(bytes.len());
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn fragmented_unicode_output_is_bounded_governed_and_byte_exact() {
    let governor = MemoryGovernor::default();
    let input = "café 学习 אבג\n".repeat(20_000);
    let mut output =
        ExportOutput::new(Writer::default(), &governor, Cancellation::default()).unwrap();
    assert_eq!(governor.usage().other, output.buffer.capacity());
    for character in input.chars() {
        write!(output, "{character}").unwrap();
    }
    output.flush().unwrap();
    let writer = output.into_inner();
    assert_eq!(writer.bytes, input.as_bytes());
    assert_eq!(writer.calls, input.len().div_ceil(BUFFER_BYTES));
    assert!(writer.max_write <= BUFFER_BYTES);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn denied_admission_and_cancelled_entry_allocate_no_output_buffer() {
    let governor = MemoryGovernor::new(GlobalMemoryLimits {
        other: Some(0),
        ..Default::default()
    });
    assert!(matches!(
        ExportOutput::new(Writer::default(), &governor, Cancellation::default()),
        Err(Failure::MemoryBudget)
    ));
    assert_eq!(governor.usage().total, 0);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(
        ExportOutput::new(Writer::default(), &governor, cancel),
        Err(Failure::Cancelled)
    ));
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn cancellation_between_partial_writes_poison_the_output_without_retrying_bytes() {
    struct Partial {
        cancel: Cancellation,
        calls: usize,
    }
    impl Write for Partial {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            self.cancel.cancel();
            Ok(bytes.len().min(3))
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("Cancelled output must not flush")
        }
    }
    let governor = MemoryGovernor::default();
    let cancel = Cancellation::default();
    let mut output = ExportOutput::new(
        Partial {
            cancel: cancel.clone(),
            calls: 0,
        },
        &governor,
        cancel,
    )
    .unwrap();
    output.write_all(b"a complete long note").unwrap();
    assert!(output.flush().is_err());
    assert!(output.flush().is_err());
    assert!(output.write_all(b"retry").is_err());
    assert_eq!(output.writer.calls, 1);
    drop(output);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn dropping_unfinished_output_never_flushes_pending_bytes() {
    let governor = MemoryGovernor::default();
    let mut writer = Writer::default();
    {
        let mut output =
            ExportOutput::new(&mut writer, &governor, Cancellation::default()).unwrap();
        output.write_all(b"incomplete export").unwrap();
    }
    assert!(writer.bytes.is_empty());
    assert_eq!(writer.calls, 0);
    assert_eq!(governor.usage().total, 0);
}

#[test]
fn interrupted_writes_retry_but_zero_writes_and_flush_errors_poison_output() {
    struct Unreliable {
        calls: usize,
        zero: bool,
        flush_error: bool,
    }
    impl Write for Unreliable {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                return Err(io::ErrorKind::Interrupted.into());
            }
            Ok(if self.zero { 0 } else { bytes.len().min(3) })
        }
        fn flush(&mut self) -> io::Result<()> {
            if self.flush_error {
                Err(io::Error::other("Synthetic flush failure"))
            } else {
                Ok(())
            }
        }
    }
    let governor = MemoryGovernor::default();
    for (zero, flush_error) in [(false, false), (true, false), (false, true)] {
        let mut output = ExportOutput::new(
            Unreliable {
                calls: 0,
                zero,
                flush_error,
            },
            &governor,
            Cancellation::default(),
        )
        .unwrap();
        output.write_all(b"complete").unwrap();
        assert_eq!(output.flush().is_ok(), !zero && !flush_error);
        let calls = output.writer.calls;
        if zero || flush_error {
            assert!(output.flush().is_err());
            assert_eq!(output.writer.calls, calls);
        }
        drop(output);
        assert_eq!(governor.usage().total, 0);
    }
}
