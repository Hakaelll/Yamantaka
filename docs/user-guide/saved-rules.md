# Manage saved rules

Open **Tools → Saved rules** to review the local rule definitions stored in your library. The native manager creates, enables, disables, and deletes definitions, and previews matching documents through a cancellable simulation. Action review resolves all six saved action kinds into an immutable plan. Confirming the plan applies its actions with persistent receipts, cancellation rollback, and reviewed undo.

## Create a definition

Choose **New rule**. Enter a name, select a condition, and enter its value when required. Select an action and enter its value, then choose **Save rule**. **Cancel editing** discards the draft.

The creation form saves one action, matching the existing Flutter form. Inherited definitions can contain up to eight actions, and the manager displays each of them. Names allow up to 120 UTF-16 units. Inherited condition and action values retain their stored lengths and use governed memory. The creation fields accept up to 32,767 characters each.

Saved condition types are document imported, source folder, file extension, author, tag, collection, exact duplicate, and unavailable file. Imported-document, exact-duplicate, and unavailable-file conditions do not require a condition value.

Saved action types are add tag, add to collection, add to workspace, reading state, favorite, and add to reading queue. Saving or enabling one does not apply its actions. Execution requires simulation, action review, and confirmation.

## Preview matching documents

Choose **Simulate** beside a supported definition. The background scan uses a read-only snapshot of the library and lists matching titles in batches of 80. It does not apply the saved actions. **Cancel simulation** stops the scan and discards the incomplete result. **Close preview** releases a completed result.

Source-folder conditions include the folder and its descendants, excluding sibling folders with a similar prefix. Author conditions use manual corrections when present. Tag and collection conditions resolve a saved ID first, then a case-normalized name when no such ID exists. Unavailable-file conditions use the stored missing flag. After a successful file or folder import, the imported-document condition uses the newly processed documents from the last completed import, including after restarting the application. **Tools → Saved rules: batch available** opens this manager. Choose **Simulate** on a rule whose condition is **Document imported**, review its matching titles, then use the normal action review and confirmation. Existing paths skipped by the importer are excluded. Other condition types keep their full-library behavior.

The manager displays the available batch count and each completed preview retains its own immutable batch. **Forget batch** removes the context for later simulations; an already reviewed preview keeps its original scope until closed. Starting another import clears the previous available batch. The completed batch is stored locally in the library journal and loaded after restart. Forgetting it also clears that persisted context. A resumed job includes documents committed during that resumed run; earlier checkpointed documents are skipped. Failed or cancelled imports do not publish a completed batch. If its full context exceeds 100,000 documents, the bounded snapshot size, or the memory budget, the PDFs can still finish importing but no partial rule context is offered. Without an available batch, this condition returns zero matches and explains how to obtain one.

Exact-duplicate conditions read full SHA-256 hashes in the background. A shared legacy fingerprint alone never confirms a duplicate. Files whose recorded identity no longer matches, or which cannot be read, are excluded and counted as unverified. A preview records what was observed during its scan; it does not authorize later changes or guarantee files remain unchanged.

Scans stop after 100,000 documents or 30 seconds, with checks between records and file chunks. Cancellation or a reached limit discards partial matches. Slow operating-system file reads may delay cancellation. Result memory uses the shared governor, and the scan does not keep the interface repainting after it settles.

## Review the planned actions

After a nonempty simulation, choose **Review actions**. The application checks that the matched documents and rule still match the simulation, then captures the current document revisions and resolves the action destinations. A changed condition or matched document requires a new simulation. Exact duplicates are read and hashed again before review; incomplete duplicate verification cannot produce a plan.

The review lists the document count, actions per document, resolved destination names, and destinations that would be created on confirmation. No destination or membership is created while reviewing. **Close review** releases the plan and returns to the matching preview.

Tag and collection actions resolve an exact saved ID first, then a normalized name. Missing tags and collections are planned as new destinations. Repeated new names share the same planned identity. Workspace and reading-queue actions require an existing workspace. Workspace names support Unicode case matching; ambiguous names require an ID.

Reading-state actions accept finished/terminado or pending/pendiente/notstarted. Priority follows the inherited favorite semantics: 0, false, and normal mean off; other nonempty values mean on. The review displays the interpreted effect. Unsupported reading-state values or unavailable workspaces reject the whole plan.

The plan stays in memory with governed allocations. Reviewing actions does not modify documents or original PDF files. Execution checks its retained document and target revisions before writing.

## Apply the reviewed plan

Choose **Confirm execution** after reviewing the document count, actions, and destinations. A background worker applies one document at a time. Every committed document change and its undo receipt share the same SQLite transaction. If a reviewed rule, destination, or document changes before its write, execution stops and attempts to revert the changes already committed.

Exact-duplicate actions reread full content and verify source revisions before each document transaction. The current source remains open against concurrent Windows writes during that transaction. Original PDF files are never modified.

**Cancel operation** stops further application and attempts to revert the committed changes. Closing the window waits for the worker acknowledgement. If current values conflict with a receipt, those values are preserved and the journal reports changes that still need review. Processed-document counts include documents subsequently reverted; they do not represent changes still present.

Tag and collection membership is added only when absent. Workspace and queue actions preserve existing entries, append up to the inherited 1,000-entry workspace limit, and retain each new entry's original page and reading mode. Finished reading state moves stored progress to the last page and records the opening time; pending resets page and progress. Favorite follows the effect shown in the review.

## Undo and recover an execution

Expand **Execution history** in the rule manager. It retrieves up to 80 native and preserved Flutter execution records at a time. Choose **Review undo**, read the explanation, and choose **Confirm undo**. Cancellation keeps the remaining receipts available for a later attempt.

Undo checks the recorded postconditions before reverting each document. It restores only changed progress/favorite fields and removes only added memberships or unchanged queue entries. Existing memberships remain intact. Newly created tags and collections are removed only when their recorded definition still matches and they are unused. Workspace revision timestamps advance when queues change.

A conflict stops that attempt and leaves the remaining receipts recoverable. Later reading progress, edited queue rows, or a reused new destination therefore require review instead of being overwritten. Reload history before retrying a stale record.

After reopening the library, unfinished native executions appear as interrupted and can be reviewed for undo. A write cannot commit without its matching receipt. This workflow does not automatically resume applying a lost in-memory plan. Inherited Flutter records are labelled separately and are not interpreted as these versioned native undo receipts.

## Review preserved Flutter executions

Execution history also lists inherited automation journals. When a journal has no saved rule name, the list uses the linked rule's current name if it is still available. Unknown receipt payloads remain stored without conversion.

For a journal with a retry/resume capability and a stored compatibility proof, choose **Simulate linked rule**. The application verifies the journal revision and compares Flutter's saved compatibility hash with the current rule's ID, condition, and ordered actions. Renaming or disabling the rule does not change that compatibility hash. Changing its condition/actions, deleting it, or using an unsupported format prevents the linked simulation.

A successful check starts a fresh simulation against the current library. Review its matches and actions, then use the normal confirmation workflow if you want a new execution. This does not silently replay a previous document offset, apply old actions, or mark the inherited journal recovered.

For a journal with undo capability, choose **Review recovery**. Known Flutter receipts support removing recorded tag, collection, and queue memberships, restoring a favorite flag, and restoring reading progress, mode, and theme. The review shows the current values and proposed restoration. Missing documents are listed without recreating them; receipts explicitly marked unchanged are skipped. Unknown kinds, extra fields, duplicate keys, and invalid values reject the review and preserve the original payload.

Flutter receipts do not record every later value. Confirming recovery therefore explicitly authorizes the displayed restoration, including replacing current favorite or reading values and removing the listed memberships. Changes made after the review invalidate its frozen revisions. Review the proposal carefully if you have edited the library since the original execution.

**Confirm recovery** runs in the background and creates a separate native recovery journal. Each document change commits with its new undo receipt. Cancellation or failure attempts to reverse the committed prefix. Only successful completion marks the inherited journal recovered. The original receipt JSON stays unchanged. A process interrupted after a document commit leaves a native journal that can be reviewed for undo after reopening.

The recovery's **Review undo** action restores the captured values, complete removed membership rows, and queue selection/split state when its postconditions still match. Later conflicting edits are preserved and leave remaining receipts recoverable. Undo also restores the inherited journal's recovery capability. No original PDF is changed. Actions absent from the old receipts cannot be inferred or recovered.

## Review existing definitions

**Enable** and **Disable** change the saved enabled flag. They preserve the original condition and action JSON. A definition whose format needs review can be disabled or removed, but cannot be newly enabled through this manager. It remains in the library with its original payload.

**Delete rule** opens an inline review. Choose **Confirm deletion** to remove the definition, or **Cancel** to keep it. Documents and operation history remain intact. If a definition changed since you loaded it, reload before changing or deleting it.

The list retrieves up to 80 definitions per batch. Previous rules and More rules appear for longer lists. Reads run through the application worker, and their retained data uses the shared memory governor. A budget error offers Reload after releasing the previous list.

See the [import-context checkpoint](../validation/rust-import-rule-context-2026-10-31.md), [receipt-recovery checkpoint](../validation/rust-legacy-recovery-2026-10-31.md), [inherited-history checkpoint](../validation/rust-inherited-journals-2026-10-31.md), [execution checkpoint](../validation/rust-rule-execution-2026-10-31.md), [action-review checkpoint](../validation/rust-rule-review-2026-10-31.md), [simulation checkpoint](../validation/rust-rule-simulation-2026-10-31.md) and [saved-rule checkpoint](../validation/rust-saved-rules-2026-10-31.md) for the executed evidence and remaining work.
