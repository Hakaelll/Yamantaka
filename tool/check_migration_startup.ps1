param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$Python,
    [string]$CaptureDirectory = "$PSScriptRoot/../build/rust-visual/migration-ui"
)
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$capturePath = (Resolve-Path -LiteralPath $CaptureDirectory).Path
$controls = Get-Content -LiteralPath (Join-Path $capturePath 'migration-selected.controls.json') -Raw | ConvertFrom-Json
if (-not $controls.synthetic -or -not $controls.legacyImport.verified -or -not $controls.legacyImport.selectedNext) {
    throw 'Expected a verified synthetic imported-library selection.'
}
$manifest = Get-Content -LiteralPath (Join-Path $capturePath 'capture-manifest.json') -Raw | ConvertFrom-Json
if ($manifest.executableSha256 -ne (Get-FileHash -LiteralPath $executablePath).Hash) { throw 'The capture executable differs.' }
$captureStores = Join-Path $repository 'build/demo-store/captures'
$sourceStores = @(Get-ChildItem -LiteralPath $captureStores -Directory | Where-Object { $_.Name -match ('^' + $controls.captureProcessId + '-[0-9a-f-]{36}$') })
if ($sourceStores.Count -ne 1) { throw 'Could not identify the owned synthetic capture store.' }
$sourceStore = $sourceStores[0].FullName
foreach ($entry in @((Get-Item -LiteralPath $sourceStore)) + @(Get-ChildItem -LiteralPath $sourceStore -Recurse -Force)) {
    if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Synthetic startup input must not contain links.' }
}
$verification = (Resolve-Path -LiteralPath (Join-Path $repository 'build/verification')).Path
$sandbox = Join-Path $verification ('migration-startup-' + [Guid]::NewGuid().ToString())
if (-not [IO.Path]::GetFullPath($sandbox).StartsWith($verification + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Unexpected sandbox directory.' }
$null = New-Item -ItemType Directory -Path $sandbox
$store = Join-Path $sandbox 'rust-data'
Copy-Item -LiteralPath $sourceStore -Destination $store -Recurse
$native = Join-Path $sandbox 'yamantaka-rust.exe'
Copy-Item -LiteralPath $executablePath -Destination $native
$null = New-Item -ItemType File -Path (Join-Path $sandbox '.yamantaka-rust-portable')
$fixtureScript = Join-Path $sandbox 'prepare_fixture.py'
@'
import json, sqlite3, sys, uuid
from pathlib import Path
root = Path(sys.argv[1]).resolve(strict=True)
assert root.name == 'rust-data'
selection = root / 'selected-library.json'
if sys.argv[2] == 'original':
    selection.write_text(json.dumps({'version': 1, 'import': None}), encoding='utf-8')
elif sys.argv[2] in ('interrupt', 'verify-recovered'):
    identity = json.loads(selection.read_text(encoding='utf-8'))['import']
    assert str(uuid.UUID(identity)) == identity
    active = root / 'imports' / identity
    journal = active.with_name('.' + identity + '.restore-journal.json')
    safety = active.with_name('pre-restore-startup')
    if sys.argv[2] == 'interrupt':
        assert active.is_dir() and not safety.exists() and not journal.exists()
        active.rename(safety)
        journal.write_text(json.dumps({'version': 1, 'active': str(active), 'safety': str(safety), 'replacement_sha256': 'not-installed'}), encoding='utf-8')
    else:
        assert active.is_dir() and not safety.exists() and not journal.exists()
else:
    database = root / 'yamantaka.sqlite3'
    assert database.is_file()
    with sqlite3.connect(database) as connection:
        connection.execute('PRAGMA foreign_keys=ON')
        connection.execute("DELETE FROM documents WHERE id<>'demo-0'")
        connection.execute("UPDATE documents SET title='Original library sentinel'")
        assert connection.execute('SELECT COUNT(*) FROM documents').fetchone()[0] == 1
'@ | Set-Content -LiteralPath $fixtureScript -Encoding UTF8
& $Python $fixtureScript $store 'prepare'
if ($LASTEXITCODE -ne 0) { throw 'Could not prepare the synthetic original library.' }
$originalHash = (Get-FileHash -LiteralPath (Join-Path $store 'yamantaka.sqlite3')).Hash
$originalPreferencesHash = (Get-FileHash -LiteralPath (Join-Path $store 'preferences.json')).Hash
$results = @()
try {
    foreach ($case in @(@{name='selected'; count=10}, @{name='recovered-import'; count=10}, @{name='original'; count=1})) {
        if ($case.name -eq 'recovered-import') {
            & $Python $fixtureScript $store 'interrupt'
            if ($LASTEXITCODE -ne 0) { throw 'Could not prepare the interrupted managed-store fixture.' }
        }
        if ($case.name -eq 'original') {
            & $Python $fixtureScript $store 'original'
            if ($LASTEXITCODE -ne 0) { throw 'Could not choose the original synthetic library.' }
        }
        $imagePath = Join-Path $sandbox ($case.name + '.png')
        $arguments = @(('"--capture={0}"' -f $imagePath))
        $process = Start-Process -FilePath $native -ArgumentList $arguments -WorkingDirectory $repository -WindowStyle Hidden -PassThru
        if (-not $process.WaitForExit(45000)) { $process.Kill(); throw "Native startup timed out: $($case.name)" }
        if ($process.ExitCode -ne 0) { throw "Native startup failed: $($case.name)" }
        $record = Get-Content -LiteralPath ([IO.Path]::ChangeExtension($imagePath,'controls.json')) -Raw | ConvertFrom-Json
        if ($record.synthetic -or -not $record.libraryQueryLoaded -or $record.libraryDocumentCount -ne $case.count) {
            throw "Production startup did not load the selected store: $($case.name)"
        }
        if ($case.name -in @('selected','recovered-import')) {
            if ((Get-FileHash -LiteralPath (Join-Path $store 'yamantaka.sqlite3')).Hash -ne $originalHash -or (Get-FileHash -LiteralPath (Join-Path $store 'preferences.json')).Hash -ne $originalPreferencesHash) {
                throw 'Imported startup changed the original store files.'
            }
        }
        if ($case.name -eq 'recovered-import') {
            & $Python $fixtureScript $store 'verify-recovered'
            if ($LASTEXITCODE -ne 0) { throw 'Managed-store startup did not complete rollback.' }
        }
        $results += [ordered]@{case=$case.name; productionStartup=$true; documentCount=$record.libraryDocumentCount; exitCode=$process.ExitCode}
        Write-Output "Production portable startup verified: $($case.name), $($case.count) books"
    }
    [ordered]@{version=1; syntheticFixture=$true; executableSha256=(Get-FileHash -LiteralPath $executablePath).Hash; cases=$results; scope='Production portable startup and next-launch selection using an isolated synthetic library. No personal store, native picker interaction, Narrator or physical DPI acceptance.'} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $sandbox 'result.json') -Encoding UTF8
    Write-Output "Result: $(Join-Path $sandbox 'result.json')"
} finally {
    # Only remove the exact executable this check copied; keep small diagnostic artifacts.
    if ((Test-Path -LiteralPath $native) -and (Get-FileHash -LiteralPath $native).Hash -eq (Get-FileHash -LiteralPath $executablePath).Hash) {
        Remove-Item -LiteralPath $native
    }
}
