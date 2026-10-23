# Compare PDFs

Open a PDF, then choose **Reading options → Compare pages**. The main reading keeps its existing tools and the reference opens with independent page and zoom controls.

## Choose a reference

The reference initially shows another page of the current PDF. Use its book button to choose another document. The chooser searches the whole catalogue, independently of the main library's filters, and retrieves pages of up to 80 books.

Protected references use the password dialog. Cancelling keeps the main reading and the earlier reference. Close the reference before changing the main document or choosing another distinct reference document.

## Arrange the views

Drag the divider to change the main reading's share of the available space. Open **Link views** to choose **Automatic**, **Side by side** or **Stacked**. Automatic stacks the panes in narrow windows.

**Swap positions** exchanges their locations while retaining each document, page, zoom and share of space.

## Link navigation

Enable **Follow page** or **Follow zoom** to follow the main reading. Manual reference changes restore independent control for that setting.

Enable **In both directions** to keep the selected page or zoom link active when changing the reference. **Fit** shares the reference's actual displayed scale with the main reading. Page numbers respect each PDF's page count.

**Follow position** also follows the point within the main PDF page. This setting always runs from the main reading toward the reference. Its canvas may limit the point that can be displayed near page edges. Manual reference page navigation turns position following off.

## Search the reference

Select the search icon in the reference header, enter a word or phrase, and press Enter or **Find**. **Match case** and **Whole words** refine the search.

Select a result to jump to its passage. Results are highlighted in the reference. **Previous** and **Next** navigate matches, including groups beyond the current batch of up to 80 results. With independent page controls, selecting a result moves only the reference. Linked navigation follows the options described above.

Use **Cancel search** to stop an active search. Closing its popup also stops a running scan; completed results remain available. A PDF without extractable text reports that condition. The main reading keeps its own query, results and selection.

## Use PDF links

Select an internal link in the reference to open its destination in that pane. Destination position and zoom are retained. Page and zoom synchronization follow your selected comparison options.

Web and email links ask for confirmation before Windows opens the destination in another application. Select **Cancel** to stay in the reader. Unsupported targets display a message without opening another application.

## Mark differences manually

In **Link views**, find **Differences · manual marks**. **Mark this page** marks the main reading's current page; **Remove mark** removes it. Matching page numbers have a thin accent border in both views.

**Previous difference** and **Next difference** jump to the nearest marked main-reading page in that direction. Whether the reference follows depends on page synchronization. These actions are disabled when no target exists.

## Save the comparison

Save the current readings through **Tools → Workspaces**. The workspace retains the reference PDF, page, canonical position, view settings, arrangement, order, following options and manual marks. Changes to a linked workspace save after a short idle interval, including closing its reference. Other saved workspace copies retain their own context.

Opening a workspace recovers the main reading before the reference. A missing or cancelled reference opening preserves the main reading. Enabled legacy split settings are adapted when there is no native comparison preference; unsupported overlay settings remain stored and produce a recoverable warning.

## Current development limits

The reference supports page navigation, zoom, text search and PDF links. Its other reader tools, per-pane keyboard shortcut routing and overlays remain in development. Release, physical accessibility and performance acceptance remain pending. See the [development checkpoints](../validation/) for the recorded scope and evidence.
