[CmdletBinding()]
param(
    [string]$OutputPath = 'build/verification/branding/verification.json',
    [string]$Python = ''
)
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if (-not $Python) {
    $bundledPython = Join-Path $env:USERPROFILE '.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
    $Python = if (Test-Path -LiteralPath $bundledPython) { $bundledPython } else { (Get-Command python -ErrorAction Stop).Source }
}
$report = if ([IO.Path]::IsPathRooted($OutputPath)) { $OutputPath } else { Join-Path $repository $OutputPath }
& $Python -X utf8 (Join-Path $PSScriptRoot 'generate_brand_assets.py') --check --report $report
if ($LASTEXITCODE) { exit $LASTEXITCODE }
