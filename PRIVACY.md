# Privacy

Yamantaka runs locally without accounts, telemetry or a cloud backend. Normal
reading and library use do not contact online services. Initial developer setup
and release downloads can use the network.

Original PDF files are read-only inputs. Annotations, reading context, metadata
corrections and organization are stored separately in the local library. PDF
passwords remain transient in memory and are not written to logs or backups.

Full-text indexing requires consent and stores extracted text locally. Diagnostic
exports use redacted fields. Review any file before sharing it. Backups contain
library data and should be treated as personal files.

Flutter migration captures an independent copy after the old application closes.
The old installation and its data remain untouched. Portable distributions store
`rust-data` and `rust-cache` beside the executable; installed-mode data uses the
independent `io.yamantaka.rust` application identity.

Folder notifications require explicit session-only authorization. They do not
import new documents or delete existing records automatically. External PDF
links require confirmation before opening a Windows handler.
