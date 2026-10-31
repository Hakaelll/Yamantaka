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

## Compatibility and current limits

The manager reads the existing v11 relation rows, including incoming links and saved source or destination anchors. New connections link whole documents. Editing anchors, anchor-specific navigation, revision groups, and rule automation remain in development.

When global indexing is enabled, creating or removing a connection updates the source document's relation search entries in the same transaction. Excluded documents remain excluded. With indexing disabled, creating a connection does not opt an unindexed document into search.

See the [development checkpoint](../validation/rust-document-connections-2026-10-31.md) for the executed evidence and remaining acceptance work.
