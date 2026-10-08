param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$native = Join-Path $root 'build\native\pdfium-7881'
$archive = Join-Path $native 'pdfium-win-x64.tgz'
$expected = '73CC0DE638AC2095E7445BF56A38200A5B7C7CA0E9F4BA144598F2457377AC08'
New-Item -ItemType Directory -Force -Path $native | Out-Null
if (!(Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest -Uri 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz' -OutFile $archive
}
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash -ne $expected) { throw 'PDFium archive hash mismatch.' }
tar -xzf $archive -C $native
if ($LASTEXITCODE -ne 0) { throw 'PDFium archive extraction failed.' }
$dll = Join-Path $native 'bin\pdfium.dll'
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $dll).Hash -ne '79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118') { throw 'PDFium DLL hash mismatch.' }
Write-Output 'PDFium 7881 x64 archive and DLL hashes verified.'
