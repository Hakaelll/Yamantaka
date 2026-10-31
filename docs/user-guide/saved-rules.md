# Manage saved rules

Open **Tools → Saved rules** to review the local rule definitions stored in your library. The native manager creates, enables, disables, and deletes definitions, and previews matching documents through a cancellable simulation. Action review resolves all six saved action kinds into an immutable plan. Confirmed application, execution cancellation, journals, and undo remain under development.

## Create a definition

Choose **New rule**. Enter a name, select a condition, and enter its value when required. Select an action and enter its value, then choose **Save rule**. **Cancel editing** discards the draft.

The creation form saves one action, matching the existing Flutter form. Inherited definitions can contain up to eight actions, and the manager displays each of them. Names allow up to 120 UTF-16 units. Inherited condition and action values retain their stored lengths and use governed memory. The creation fields accept up to 32,767 characters each.

Saved condition types are document imported, source folder, file extension, author, tag, collection, exact duplicate, and unavailable file. Imported-document, exact-duplicate, and unavailable-file conditions do not require a condition value.

Saved action types are add tag, add to collection, add to workspace, reading state, favorite, and add to reading queue. These are definitions for the execution workflow being connected next. Saving or enabling one does not apply its actions to library documents in this checkpoint.

## Preview matching documents

Choose **Simulate** beside a supported definition. The background scan uses a read-only snapshot of the library and lists matching titles in batches of 80. It does not apply the saved actions. **Cancel simulation** stops the scan and discards the incomplete result. **Close preview** releases a completed result.

Source-folder conditions include the folder and its descendants, excluding sibling folders with a similar prefix. Author conditions use manual corrections when present. Tag and collection conditions resolve a saved ID first, then a case-normalized name when no such ID exists. Unavailable-file conditions use the stored missing flag. A manual simulation has no import batch, so the imported-document condition returns no matches. Import-triggered execution is still pending.

Exact-duplicate conditions read full SHA-256 hashes in the background. A shared legacy fingerprint alone never confirms a duplicate. Files whose recorded identity no longer matches, or which cannot be read, are excluded and counted as unverified. A preview records what was observed during its scan; it does not authorize later changes or guarantee files remain unchanged.

Scans stop after 100,000 documents or 30 seconds, with checks between records and file chunks. Cancellation or a reached limit discards partial matches. Slow operating-system file reads may delay cancellation. Result memory uses the shared governor, and the scan does not keep the interface repainting after it settles.

## Review the planned actions

After a nonempty simulation, choose **Review actions**. The application checks that the matched documents and rule still match the simulation, then captures the current document revisions and resolves the action destinations. A changed condition or matched document requires a new simulation. Exact duplicates are read and hashed again before review; incomplete duplicate verification cannot produce a plan.

The review lists the document count, actions per document, resolved destination names, and destinations that would be created on confirmation. No destination or membership is created while reviewing. **Close review** releases the plan and returns to the matching preview.

Tag and collection actions resolve an exact saved ID first, then a normalized name. Missing tags and collections are planned as new destinations. Repeated new names share the same planned identity. Workspace and reading-queue actions require an existing workspace. Workspace names support Unicode case matching; ambiguous names require an ID.

Reading-state actions accept finished/terminado or pending/pendiente/notstarted. Priority follows the inherited favorite semantics: 0, false, and normal mean off; other nonempty values mean on. The review displays the interpreted effect. Unsupported reading-state values or unavailable workspaces reject the whole plan.

The plan stays in memory with governed allocations. Its document and target revisions are retained for the execution boundary being connected next. **Confirmed execution, its journal, recovery, and undo are not available yet.** Reviewing actions does not modify documents or original PDF files.

## Review existing definitions

**Enable** and **Disable** change the saved enabled flag. They preserve the original condition and action JSON. A definition whose format needs review can be disabled or removed, but cannot be newly enabled through this manager. It remains in the library with its original payload.

**Delete rule** opens an inline review. Choose **Confirm deletion** to remove the definition, or **Cancel** to keep it. Documents and operation history remain intact. If a definition changed since you loaded it, reload before changing or deleting it.

The list retrieves up to 80 definitions per batch. Previous rules and More rules appear for longer lists. Reads run through the application worker, and their retained data uses the shared memory governor. A budget error offers Reload after releasing the previous list.

See the [action-review checkpoint](../validation/rust-rule-review-2026-10-31.md), [simulation checkpoint](../validation/rust-rule-simulation-2026-10-31.md) and [saved-rule checkpoint](../validation/rust-saved-rules-2026-10-31.md) for the executed evidence and remaining work.
