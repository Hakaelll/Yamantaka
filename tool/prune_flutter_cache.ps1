[CmdletBinding()]
param([switch]$Apply, [switch]$IncludeSdk)
$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$prefix = $repository.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
if (Get-Process -Name dart, dartaotruntime, flutter -ErrorAction SilentlyContinue) {
    throw 'Close Flutter and Dart tools before cleaning their generated files.'
}
$names = @(
    '.dart_tool', '.flutter-plugins-dependencies', '.pub', '.pub-cache', 'coverage',
    'third_party/pdfrx/.dart_tool', 'third_party/pdfrx/.flutter-plugins-dependencies',
    'third_party/pdfrx/pubspec.lock', 'windows/flutter/ephemeral', 'build/native_assets'
)
if ($IncludeSdk) { $names += 'build/tools/flutter' }
$candidates = @()
foreach ($name in $names) {
    $target = [IO.Path]::GetFullPath((Join-Path $repository $name))
    if (-not $target.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Cleanup target escaped the workspace.'
    }
    $ancestor = $target
    while ($ancestor.StartsWith($repository, [StringComparison]::OrdinalIgnoreCase)) {
        if ((Test-Path -LiteralPath $ancestor) -and
            ((Get-Item -LiteralPath $ancestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'Cleanup refuses linked workspace paths.'
        }
        if ($ancestor.Equals($repository, [StringComparison]::OrdinalIgnoreCase)) { break }
        $ancestor = [IO.Path]::GetDirectoryName($ancestor)
    }
    if (-not (Test-Path -LiteralPath $target)) { continue }
    $tracked = @(git -C $repository ls-files -- $name)
    if ($LASTEXITCODE -ne 0 -or $tracked.Count -gt 0) {
        throw 'Cleanup refuses tracked or unverifiable files.'
    }
    $item = Get-Item -LiteralPath $target -Force
    $children = if ($item.PSIsContainer) { @(Get-ChildItem -LiteralPath $target -Force -Recurse) } else { @($item) }
    if ($children | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }) {
        throw 'Cleanup refuses directories containing links.'
    }
    $bytes = [long](($children | Where-Object { -not $_.PSIsContainer } | Measure-Object Length -Sum).Sum)
    $candidates += [pscustomobject]@{ path = $name; bytes = $bytes }
}
foreach ($candidate in $candidates) {
    if ($Apply) {
        Remove-Item -LiteralPath (Join-Path $repository $candidate.path) -Recurse -Force
    }
}
[pscustomobject]@{
    applied = [bool]$Apply
    includedSdk = [bool]$IncludeSdk
    entries = $candidates
    candidateBytes = [long](($candidates | Measure-Object bytes -Sum).Sum)
    remainingEntries = @($candidates | Where-Object { Test-Path -LiteralPath (Join-Path $repository $_.path) } | Select-Object -ExpandProperty path)
}
