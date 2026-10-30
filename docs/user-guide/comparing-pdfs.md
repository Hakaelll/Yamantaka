# Compare PDFs

Open a PDF, then choose **Reading options → Compare pages**. The main reading keeps its existing tools and the reference opens with independent page and zoom controls. In smaller panes, **Reference options** groups the book chooser, zoom and linking controls to leave more room for the PDF.

## Choose a reference

The reference initially shows another page of the current PDF. Use its book button to choose another document. The chooser searches the whole catalogue, independently of the main library's filters, and retrieves pages of up to 80 books.

Protected references use the password dialog. Cancelling keeps the main reading and the earlier reference. Close the reference before changing the main document or choosing another distinct reference document.

## Arrange the views

Drag the divider to change the main reading's share of the available space. Open **Link views** to choose **Automatic**, **Side by side** or **Stacked**. Automatic stacks the panes in narrow windows.

**Swap positions** exchanges their locations while retaining each document, page, zoom and share of space.

## Choose the keyboard target

Click a pane or one of its controls to make it active. Keyboard focus entering a pane's controls also selects that pane. A thin accent border identifies the active reading. **F6** switches between the two reading canvases and closes transient popups. In a narrow window, switching to the reference hides a full-width main navigation drawer while retaining its contents.

| Shortcut | Active-pane action |
| --- | --- |
| Left / Right, Page Up / Page Down | Previous / next page when the canvas has keyboard input |
| Ctrl++ / Ctrl+- | Zoom in / out using the existing reader zoom controls |
| Ctrl+0 | Fit page |
| Alt+Left / Alt+Right | Previous / next history position when a canvas has keyboard input |
| Ctrl+F | Open that PDF's search and focus its query field |
| F3 / Shift+F3 | Next / previous result in that PDF |
| Ctrl+W | Close the reference when it is active; otherwise close the main reading tab |

Arrow keys stay with an input field while editing. Page keys also suppress the UI toolkit's spatial focus movement, so paging a left-hand reference keeps subsequent commands in that pane. The main reading's text-selection shortcuts are inactive while the reference owns keyboard input.

Fullscreen (**F11**), Focus mode (**F8**) and global search (**Ctrl+Shift+F**) retain their application-wide behavior. Page and zoom commands still respect your comparison synchronization choices.

## Link navigation

Enable **Follow page** or **Follow zoom** to follow the main reading. Manual reference changes restore independent control for that setting.

Enable **In both directions** to keep the selected page or zoom link active when changing the reference. **Fit** shares the reference's actual displayed scale with the main reading. Page numbers respect each PDF's page count.

**Follow position** also follows the point within the main PDF page. This setting always runs from the main reading toward the reference. Its canvas may limit the point that can be displayed near page edges. Manual reference page navigation turns position following off.

Open **Navigate reference PDF** in the reference header to choose a section. Use the section selector at the top of the popup.

## Return to reading positions

Open **Navigate reference PDF → Positions** to move backward or forward through the reference's navigation history. **Alt+Left** and **Alt+Right** do the same in the active reading. Input fields retain these keys. A new page jump after going back replaces the forward branch.

Each A/B row has a bookmark icon labelled **Save position A** or **Save position B**. Use it to capture that reading's current page and position. Select **Go to A** or **Go to B** to return. The two readings keep independent marks, including when they show the same PDF. Marks are temporary reading positions; use **Bookmarks** for saved document bookmarks.

The main navigator also includes **Positions**, and **Reading options → Positions** provides the same controls. The main navigator's selected section is retained by saved reading contexts. Small windows keep the controls in a scrollable menu. History navigation and A/B jumps respect your comparison synchronization settings.

## Browse reference contents

Open **Navigate reference PDF → Contents** in the reference header to browse its chapter hierarchy. Use the plus or minus controls to expand or collapse sections, then select a chapter or passage. Its destination page, position and zoom apply to the reference; synchronization follows your selected options.

The popup reports when the PDF has no contents and offers retry after a loading error. Closing it cancels an outstanding request. The main contents panel retains its own state.

## Browse page thumbnails

Open **Navigate reference PDF → Pages** to browse thumbnails. Select a page to navigate the reference. The main reading retains its own page unless page synchronization is enabled.

Only visible thumbnails are rendered, with at most 16 resident slots and a maximum image side of 256 physical pixels. Closing the navigator or switching sections releases the thumbnails and cancels pending work.

## Save reference bookmarks

Open **Navigate reference PDF → Bookmarks**, then select **Bookmark page N**. Select a saved page to return to it. **Edit** changes its label; **Remove** deletes that bookmark. Lists retrieve up to 80 entries per page, with Previous and Next controls for longer lists.

Bookmarks belong to their PDF. Distinct documents keep separate lists. When both panes show the same PDF, changes appear in both lists while each pane keeps its own navigation and editor. Selecting a bookmark follows the comparison's page synchronization options.

Closing the navigator cancels an outstanding list query. An accepted save, rename or removal continues if you close the reference. A failed write remains visible in the owning editor, or in the main application if that editor has closed. Changing bookmarks also refreshes library results, including the Bookmarked view.

## Read and copy page text

Open **Navigate reference PDF → Text** to read the current reference page's extracted text. Change the reference page through its page field or another navigation tool; the text follows that page. **Copy text** copies the page's available text to the clipboard.

Image-only pages display a message when no text can be extracted. Extraction errors offer **Retry**. Closing the navigator or switching sections cancels its pending request and releases the reference text. The main reading keeps its own text and selection state.

## Search the reference

Open **Navigate reference PDF → Find**, enter a word or phrase, and press Enter or **Find**. **Match case** and **Whole words** refine the search.

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

The reference supports page navigation, history, temporary A/B positions, zoom, chapter contents, page thumbnails, bookmarks, page text and copying, text search and PDF links. Reference text selection and annotation tools, shortcut customization, overlays and broader keyboard acceptance remain in development. Release, physical accessibility and performance acceptance remain pending. See the [development checkpoints](../validation/) for the recorded scope and evidence.
