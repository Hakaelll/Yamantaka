use std::io::{self, Read, Write};
use windows::{
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::DataWriter,
    Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
};

fn main() -> std::process::ExitCode {
    match recognize() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn recognize() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args_os()
        .nth(1)
        .ok_or("An image file is required")?;
    // SAFETY: this standalone process initializes WinRT on its main thread.
    unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            // SAFETY: this guard is dropped on the initialized main thread.
            unsafe { RoUninitialize() };
        }
    }
    let _apartment = Apartment;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|_| "Install a Windows OCR language for your user profile")?;
    let mut file = std::fs::File::open(input)?;
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    let width = u32::from_le_bytes(header[4..8].try_into()?);
    let height = u32::from_le_bytes(header[8..12].try_into()?);
    let limit = OcrEngine::MaxImageDimension()?.min(3000);
    if &header[..4] != b"YOCR" || !(1..=limit).contains(&width) || !(1..=limit).contains(&height) {
        return Err("Invalid grayscale OCR raster".into());
    }
    let mut pixels = vec![0; (width * height) as usize];
    file.read_exact(&mut pixels)?;
    let buffer = DataWriter::new()?;
    buffer.WriteBytes(&pixels)?;
    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer.DetachBuffer()?,
        BitmapPixelFormat::Gray8,
        width as i32,
        height as i32,
    )
    .map_err(|error| format!("OCR raster: {error}"))?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .and_then(|operation| operation.join())
        .map_err(|error| format!("Windows recognizer: {error}"))?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (line, words) in result.Lines()?.into_iter().enumerate() {
        for word in words.Words()? {
            let bounds = word.BoundingRect()?;
            let text = word
                .Text()?
                .to_string_lossy()
                .replace(['\t', '\r', '\n'], " ");
            writeln!(
                output,
                "{}\t{}\t{}\t{}\t{}\t{text}",
                bounds.X, bounds.Y, bounds.Width, bounds.Height, line
            )?;
        }
    }
    output.flush()?;
    Ok(())
}
