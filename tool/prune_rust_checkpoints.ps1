param([switch]$Apply)
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$directory = (Resolve-Path -LiteralPath (Join-Path $repository 'build/verification')).Path
$expected = [IO.Path]::GetFullPath((Join-Path $repository 'build/verification'))
if (-not $directory.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) { throw 'Unexpected verification directory.' }
foreach ($ancestor in @($repository, (Join-Path $repository 'build'), $directory)) {
    if ((Get-Item -LiteralPath $ancestor).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'Checkpoint cleanup requires ordinary workspace directories.'
    }
}
$tracked = @(git -C $repository ls-files -- build/verification)
if ($LASTEXITCODE -ne 0 -or $tracked.Count -gt 0) { throw 'Refusing to prune a tracked or unverifiable directory.' }
$running = @()
foreach ($process in @(Get-CimInstance -ClassName Win32_Process -Filter "Name LIKE 'yamantaka%.exe'" -ErrorAction Stop)) {
    if (-not $process.ExecutablePath) { throw 'A running Yamantaka path could not be inspected; no files removed.' }
    $running += [IO.Path]::GetFullPath($process.ExecutablePath)
}
$files = @(Get-ChildItem -LiteralPath $directory -File | Where-Object {
    $_.Name -match '^yamantaka-[a-z0-9-]+\.exe$' -and -not ($_.Attributes -band [IO.FileAttributes]::ReparsePoint)
} | Sort-Object LastWriteTimeUtc -Descending)
$keep = @($files | Select-Object -First 2 -ExpandProperty FullName) + $running
$candidates = @()
foreach ($file in $files) {
    $path = [IO.Path]::GetFullPath($file.FullName)
    if (-not ([IO.Path]::GetDirectoryName($path)).Equals($directory, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'A checkpoint escaped its intended directory.'
    }
    if ($keep -notcontains $path) { $candidates += $file }
}
$removed = @()
$skipped = @()
foreach ($file in $candidates) {
    if (-not $Apply) { continue }
    try {
        $stream = [IO.File]::Open($file.FullName, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
        try {
            if ($stream.ReadByte() -ne 77 -or $stream.ReadByte() -ne 90) { throw 'Not a native executable.' }
        } finally { $stream.Dispose() }
        $current = Get-Item -LiteralPath $file.FullName
        if ($current.Length -ne $file.Length -or $current.LastWriteTimeUtc -ne $file.LastWriteTimeUtc -or
            ($current.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Checkpoint changed during inspection.' }
        Remove-Item -LiteralPath $file.FullName -ErrorAction Stop
        $removed += $file
    } catch {
        $skipped += $file.Name
    }
}
[pscustomobject]@{
    applied = [bool]$Apply
    scannedFiles = $files.Count
    candidateFiles = $candidates.Count
    candidateBytes = [long](($candidates | Measure-Object Length -Sum).Sum)
    removedFiles = $removed.Count
    removedBytes = [long](($removed | Measure-Object Length -Sum).Sum)
    keptNames = @($files | Where-Object { $keep -contains $_.FullName } | Select-Object -ExpandProperty Name)
    skippedNames = $skipped
}
