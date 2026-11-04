[CmdletBinding()]
param([string[]]$AppArguments = @())
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_BUILD_JOBS = '2'
Push-Location -LiteralPath $repository
try {
    & cargo run --locked --offline -p yamantaka --bin yamantaka-rust -- @AppArguments
    $result = $LASTEXITCODE
} finally { Pop-Location }
exit $result
