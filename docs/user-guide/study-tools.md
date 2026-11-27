# Study with notebooks and passage links

- Open **Study** and use **Study actions** to open notebooks. Select a passage and use **More actions → Add to notebook** to save a quote. In Notes, **Add to notebook** also accepts a note or a linked crop. Reorder the entries, save your changes, and export the saved notebook to Markdown. **Go to passage** opens the first page of the entry and shows its location.
- In comparison, select a passage in either pane and choose **More actions → Link passage**. Repeat in the other pane, add an observation, and save. **Open comparison** reopens the saved PDFs at the two passages. Attachments are checked against the stored document revision before they are shown.
- Choose **Marked PDF copy**, choose a page range, and use **Preview** to review the first page with marks in the reader. **Save copy** creates a new PDF with the attached highlights, underlines, and drawings in that range. The source PDF and any existing destination file stay intact.

## Enable optional local OCR

Build the Windows helper separately from Yamantaka:

```powershell
./tool/package_study_ocr.ps1
```

Extract the generated archive beside the Yamantaka executable. Keep the helper at `ocr/yamantaka-ocr.exe`. Enable an OCR language in Windows for your user profile. Choose **Study actions → Recognize text · OCR** and select the current page or up to 24 pages. Use **Select** and **Find** on the recognized text. Cancel from the dialog while recognition is running.

The recognized text stays in memory while the document is open. Saved highlights, notes, notebooks and links use the existing library and backup formats. On reopening, run OCR again to select or search scanned text.

The add-on depends on the local Windows recognizer. Microsoft documents desktop support for apps with package identity. Check whether the engine is available on your Windows installation. [Windows OCR API](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr)
