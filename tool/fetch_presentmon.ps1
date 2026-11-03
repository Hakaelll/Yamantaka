[CmdletBinding()]
param([switch]$VerifyOnly)
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$directory = Join-Path $repository 'build/native/presentmon-2.6.0'
$expected = 'b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af'
$url = 'https://github.com/GameTechDev/PresentMon/releases/download/v2.6.0/PresentMon-2.6.0-x64.exe'
foreach ($ancestor in @($repository, (Join-Path $repository 'build'), (Join-Path $repository 'build/native'), $directory)) {
    if ((Test-Path -LiteralPath $ancestor) -and ((Get-Item -LiteralPath $ancestor).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'PresentMon cache requires ordinary workspace directories.'
    }
}
$executable = Join-Path $directory 'PresentMon-2.6.0-x64.exe'
if (-not (Test-Path -LiteralPath $executable)) {
    if ($VerifyOnly) { throw 'Fetch the pinned portable PresentMon executable first.' }
    New-Item -ItemType Directory -Force -Path $directory | Out-Null
    $temporary = Join-Path $directory ('.download-' + [guid]::NewGuid().ToString('N'))
    try {
        Invoke-WebRequest -Uri $url -OutFile $temporary
        if ((Get-FileHash -LiteralPath $temporary -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
            throw 'PresentMon download differs from the pinned official asset.'
        }
        Move-Item -LiteralPath $temporary -Destination $executable
    } finally {
        if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary }
    }
}
if ((Get-Item -LiteralPath $executable).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Unexpected PresentMon link.' }
if ((Get-FileHash -LiteralPath $executable -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Cached PresentMon differs from the pinned official asset; file preserved.'
}
[pscustomobject]@{ version = '2.6.0'; executable = $executable; sha256 = $expected; source = $url } | ConvertTo-Json
