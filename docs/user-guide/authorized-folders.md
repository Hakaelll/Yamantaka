# Authorize folder notifications

This source change follows the review 11 candidate. It is not included in the
published review 10 download or the existing review 11 bundle.

Open **Settings → Storage → Authorized folders**, then choose **Authorize
folder…** and select a directory. Yamantaka listens for changes in that directory
and its subfolders until you close the application. Up to eight roots can be
authorized per session. The permissions and directory paths are not saved.

**Pause** stops catalogue refreshes from all authorized roots. **Resume** reloads
the saved catalogue once and resumes notifications. **Revoke permission** closes
the selected notification handle. An unavailable root offers **Retry**; failures
do not reveal filesystem details in diagnostic messages. Safe mode disables
folder authorization.

Notifications reload existing catalogue records. They do not scan or import new
PDFs, delete records, move files, repair paths, or alter originals. Use **Import
folder…** to add books and the existing file-check or relocation tools to review
changed or missing sources. Visible covers retain their separate source monitor.

The Windows notification worker waits without polling at idle and coalesces
changes after 350 ms of quiet. Revocation signals its cancellation event before
joining the worker. A full application queue retains the requested refresh for
retry rather than silently dropping it.

Windows can omit notifications for remote filesystems and does not report a
change to the watched root itself. Symbolic links inside a watched tree do not
extend observation to their target files. These are
[Windows notification semantics](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstchangenotificationw).
Folder selection and notification setup can still depend on filesystem or
network responsiveness.

The implementation follows Flutter's connected folder-watch callback, which
reloads the library without importing or deleting documents. No new runtime
tests or rendered captures were executed for this change at the owner's request.
