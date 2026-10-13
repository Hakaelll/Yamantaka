param(
    [string]$SourceDirectory = "$PSScriptRoot/../build/rust-visual/stage2",
    [string]$DestinationDirectory = "$PSScriptRoot/../docs/screenshots/rust-stage2",
    [string]$Impeccable = "$env:USERPROFILE/.codex/skills/impeccable/scripts/impeccable.cmd"
)
$ErrorActionPreference = 'Stop'
$sourcePath = (Resolve-Path -LiteralPath $SourceDirectory).Path
$null = New-Item -ItemType Directory -Path $DestinationDirectory -Force
$destinationPath = (Resolve-Path -LiteralPath $DestinationDirectory).Path
$manifest = Get-Content -LiteralPath (Join-Path $sourcePath 'capture-manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if (-not $manifest.synthetic) { throw 'Refusing to publish evidence from a personal store.' }
foreach ($capture in $manifest.captures) {
    $originalImage = Join-Path $sourcePath $capture.image
    $originalControls = Join-Path $sourcePath $capture.controls
    if ((Get-FileHash -LiteralPath $originalImage -Algorithm SHA256).Hash -ne $capture.imageSha256 -or
        (Get-FileHash -LiteralPath $originalControls -Algorithm SHA256).Hash -ne $capture.controlsSha256) {
        throw "Evidence changed since capture: $($capture.name)"
    }
    $publishedImage = Join-Path $destinationPath $capture.image
    Copy-Item -LiteralPath $originalImage -Destination $publishedImage
    Copy-Item -LiteralPath $originalControls -Destination (Join-Path $destinationPath $capture.controls)
    $originPath = Join-Path $destinationPath "$($capture.name).origin.txt"
    @"
Origin: native screenshot of the Yamantaka Rust executable, not an AI-generated image.
Captured with tool/capture_rust_ui.ps1; isolated synthetic study collection; PDF source is the repository's generated acceptance fixture. Write scenarios may use an isolated file-backed synthetic store. No personal library was opened.
Case: $($capture.name)
Arguments: $($capture.args -join ' ')
Capture UTC: $($manifest.capturedAtUtc)
Executable SHA-256: $($manifest.executableSha256)
Original PNG SHA-256: $($capture.imageSha256)
Logical viewport: $($capture.viewport -join ', ')
Pixels per logical point: $($capture.pixelsPerPoint)
See capture-manifest.json for exact source hashes and conditions. These captures do not certify physical monitor DPI, Narrator, full functional parity or whole-rewrite acceptance.
"@ | Set-Content -LiteralPath $originPath -Encoding UTF8
    & $Impeccable embed-prompt $publishedImage --prompt-file $originPath
    if ($LASTEXITCODE -ne 0) { throw "Could not embed screenshot origin: $($capture.name)" }
    $capture | Add-Member -NotePropertyName publishedImageSha256 -NotePropertyValue (Get-FileHash -LiteralPath $publishedImage -Algorithm SHA256).Hash
}
$manifest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $destinationPath 'capture-manifest.json') -Encoding UTF8
& $Impeccable embed-prompt --scan $destinationPath
if ($LASTEXITCODE -ne 0) { throw 'Screenshot provenance scan failed.' }
