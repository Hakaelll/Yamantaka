[CmdletBinding()]
param(
    [string]$OutputDirectory = 'build/release/yamantaka-0.17.0',
    [string]$Cargo = 'cargo',
    [string]$Python = 'python',
    [switch]$Portable,
    [switch]$VerifyOnly
)
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'package_rust_windows.ps1') @PSBoundParameters
if ($LASTEXITCODE) { exit $LASTEXITCODE }
