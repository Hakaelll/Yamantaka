[CmdletBinding()]
param(
    [string]$OutputDirectory = 'build/release/yamantaka-0.16.0',
    [string]$Cargo = 'cargo',
    [string]$Python = 'python',
    [switch]$Portable,
    [switch]$VerifyOnly
)
$ErrorActionPreference='Stop'
$repository=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output=[IO.Path]::GetFullPath((Join-Path $repository $OutputDirectory))
$releaseRoot=[IO.Path]::GetFullPath((Join-Path $repository 'build/release'))
if (-not $output.StartsWith($releaseRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Use a named output directory below build/release.'}
function Test-BundleResources {
    $executable = Join-Path $output 'yamantaka/yamantaka.exe'
    & $Python -X utf8 (Join-Path $PSScriptRoot 'verify_windows_icon.py') $executable --report (Join-Path $output 'icon-verification.json')
    if ($LASTEXITCODE) {throw 'Executable icon verification failed.'}
    $actualVersion = (Get-Item -LiteralPath $executable).VersionInfo.ProductVersion
    $manifest = Get-Content -LiteralPath (Join-Path $output 'package-manifest.json') -Raw | ConvertFrom-Json
    if ($actualVersion -ne $manifest.version) {throw 'Executable version differs from the package manifest.'}
    Write-Output ('Native Windows bundle verified: '+$manifest.version)
}
if ($VerifyOnly) {
    & $Python -X utf8 (Join-Path $PSScriptRoot 'rust_windows_bundle.py') $output --verify-only
    if ($LASTEXITCODE) {exit $LASTEXITCODE}
    Test-BundleResources
    return
}
if (Test-Path -LiteralPath $output) {throw 'The destination exists. Select a fresh review directory.'}
$packageArguments=@('-X','utf8',(Join-Path $PSScriptRoot 'rust_windows_bundle.py'),$output,'--cargo',$Cargo)
if ($Portable) {$packageArguments+='--portable'}
& $Python @packageArguments
if ($LASTEXITCODE) {exit $LASTEXITCODE}
Test-BundleResources
