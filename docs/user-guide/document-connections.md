# Connect documents

Open a book's details and choose **Document connections**. This screen shows connections created from that document and incoming connections from other documents.

## Add a connection

Choose **Choose document** to search the whole catalogue by title, author, or filename. Previous and Next browse batches of up to 80 documents. The current document cannot be selected as its own destination. A missing PDF can still participate in library connections.

Select a connection type, then choose **Add connection**:

| Type | Meaning from the current document | Incoming label |
| --- | --- | --- |
| Related | A companion reading | Related |
| Replaces | A newer or alternative edition replaces the destination | Replaced by |
| Source of | The current document is a source for the destination | Source |
| Attachment of | The current document accompanies the destination | Has attachment |

Repeating the same source, destination, and type reuses the existing connection. Opposite directions remain distinct. Incoming labels describe the original connection without creating an inverse row.

## Follow and remove connections

**Open document** opens the other reading through the normal reader. **View connections** keeps you in the manager and shows the other document's connections.

**Remove connection** opens an inline review. Choose **Confirm removal** to delete that connection, or **Cancel** to keep it. Both PDF files and their library records remain intact. If the connection changed elsewhere, reload before trying again.

The list retrieves up to 80 connections per batch. Previous and Next retrieve further batches. Changes run locally in the background, and the manager keeps the previous list until its refresh arrives. Failed operations show a retry path.

## Group editions

Choose **Revision groups** in the manager's section menu, then **New group**. Enter a **Group name**, choose **Choose another edition**, and select a document from the catalogue. **Create group** saves the current document and that edition together, with the current document marked as the group's current edition.

Choose **View editions** on a group to load its members. **Open edition** opens a member through the normal reader. **Make current** changes which member is current; every document in that group shares the same choice. An unavailable PDF stays in its group but cannot be opened until its source is available again.

Existing groups keep their identifiers, names, membership, and current choice. The list retrieves up to 80 groups per batch, and each opened group retrieves up to 80 members per batch. Use the corresponding previous or more controls to continue. The creation form starts a group with two documents; existing groups with more members remain readable.

A group that changed since it was loaded must be reloaded before another current-edition change. If its current document is removed from the library, the group remains and you can choose a surviving member. Creating a group does not move, replace, merge, or modify its original PDFs.

## Compatibility and current limits

The manager reads the existing v11 relation rows, including incoming links and saved source or destination anchors. New connections link whole documents. Editing anchors, anchor-specific navigation, and rule automation remain in development. The revision manager creates groups and changes their current edition; group renaming, removal, and membership editing are not provided by this flow.

When global indexing is enabled, creating or removing a connection updates the source document's relation search entries in the same transaction. Excluded documents remain excluded. With indexing disabled, creating a connection does not opt an unindexed document into search.

See the [connections checkpoint](../validation/rust-document-connections-2026-10-31.md) and [revision-group checkpoint](../validation/rust-revision-groups-2026-10-31.md) for the executed evidence and remaining acceptance work.
