# Two-reader admission checkpoint

Opening confirmation now accepts the complete visible document set atomically. It retains at most two unique documents, requires the newly opened document, and rejects references to documents outside the previous visible set. Cancellation restores the previous set; stale confirmations remain harmless. The single-reader UI keeps its existing behavior.

The existing native import-preemption scenario now admits its second reader with both document IDs in the confirmation, without a later visibility repair. The focused run passed: `cargo test -p yamantaka-application --test import_resumption_runtime native_reader_preemption_resumes_the_interrupted_page_and_keeps_prior_evidence -- --exact`. It exercises two protected readers while interrupted PDF analysis waits and subsequently resumes.

This checkpoint supplies the ownership boundary for native split reading. A second viewport, pane-specific interactions, synchronization and saved split restoration remain pending; no split UI or complete parity is claimed.
