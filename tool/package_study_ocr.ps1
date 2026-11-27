param([string]$Cargo = 'cargo')
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$target = Join-Path $root 'target/study-ocr-bundle'
$output = Join-Path $root 'build/release/yamantaka-study-ocr'
$archive = Join-Path $output 'yamantaka-study-ocr-windows-x64.zip'
if (Test-Path -LiteralPath $output) { throw 'Choose a fresh output by moving the previous add-on package.' }
$previousFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    $env:CARGO_ENCODED_RUSTFLAGS = @('-C', 'target-feature=+crt-static', "--remap-path-prefix=$root=/_yamantaka", "--remap-path-prefix=$env:USERPROFILE=/_user") -join [char]31
    & $Cargo build --release --locked --offline --manifest-path (Join-Path $root 'Cargo.toml') --target x86_64-pc-windows-msvc --target-dir $target -p yamantaka-ocr
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previousFlags
}
$entry = (New-Item -ItemType Directory -Path (Join-Path $output 'ocr')).FullName
Copy-Item -LiteralPath (Join-Path $target 'x86_64-pc-windows-msvc/release/yamantaka-ocr.exe') -Destination $entry
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $output
@'
Yamantaka optional OCR add-on for Windows x64

Extract the ocr folder next to the Yamantaka executable. Enable an OCR language in Windows.
Use Study actions to recognize the current page or a page range. Recognition uses the local Windows engine.
'@ | Set-Content -LiteralPath (Join-Path $output 'README.txt') -Encoding utf8
Compress-Archive -LiteralPath $entry,(Join-Path $output 'LICENSE'),(Join-Path $output 'README.txt') -DestinationPath $archive
$hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $([IO.Path]::GetFileName($archive))" | Set-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt') -Encoding ascii
Get-Item -LiteralPath $archive
