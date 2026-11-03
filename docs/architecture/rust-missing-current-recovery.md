# Recovery with no current directory

Pending-operation review now distinguishes an absent active directory from an
existing directory with a frozen tree hash. Reading the operation records and
reviewing a previous library do not create a replacement directory. The selected
previous store must remain a validated `pre-restore-…` sibling with unchanged
content. A directory appearing or disappearing after review invalidates the
choice.

Normal rollback review still requires an existing active directory. The separate
pending-recovery path can construct a rollback review for an absent destination.
It records the absence explicitly and skips the original-to-safety move. The
recovery view reports that no current folder is available for a safety copy.

Existing rollback journals retain their version-one encoding. Version two is
reserved for the recorded `original_existed: false` state. Before installation,
the source and absent destination can resume the selected move after source
hash and integrity validation. After installation, a matching active folder and
absent source complete journal cleanup. Unexpected extra folders or changed
content remain unresolved; recovery does not overwrite or remove them.

A failed move with no original directory retains the journal for a later retry.
If post-install validation fails, the source is returned when its old location
is still free, and the unresolved journal remains. Pending operation records
continue to be archived separately before this rollback transaction starts.

The native application builds with this implementation. Functional, interruption
and visual acceptance of the missing-directory path remain pending; no new test
functions or executed scenario results are claimed here.
