param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$Python,
    [string]$CaptureDirectory="$PSScriptRoot/../build/rust-visual/workspace-recovery"
)
$ErrorActionPreference='Stop'
. "$PSScriptRoot/wait_rust_test_process.ps1"
$repository=(Resolve-Path -LiteralPath "$PSScriptRoot/..").Path
$executablePath=(Resolve-Path -LiteralPath $Executable).Path
$capturePath=(Resolve-Path -LiteralPath $CaptureDirectory).Path
$record=Get-Content -LiteralPath (Join-Path $capturePath 'workspaces.controls.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$manifest=Get-Content -LiteralPath (Join-Path $capturePath 'capture-manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
if (-not $record.synthetic -or $record.workspaces.bindingName -ne 'Reference copy' -or -not $record.workspaces.reference) {throw 'Expected the synthetic bound-workspace capture.'}
if ($manifest.executableSha256 -ne (Get-FileHash -LiteralPath $executablePath).Hash) {throw 'Capture and executable differ.'}
$captures=Join-Path $repository 'build/demo-store/captures'
$sources=@(Get-ChildItem -LiteralPath $captures -Directory | Where-Object {$_.Name -match ('^'+$record.captureProcessId+'-[0-9a-f-]{36}$')})
if ($sources.Count -ne 1) {throw 'Could not identify the owned synthetic store.'}
$source=$sources[0].FullName
foreach ($path in @($source,(Join-Path $source 'export-preview.sqlite3'),(Join-Path $source 'export-preview.reader-continuity.json'))) {
    if ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint) {throw 'Synthetic inputs must be ordinary files and directories.'}
}
$verification=(Resolve-Path -LiteralPath (Join-Path $repository 'build/verification')).Path
$sandbox=Join-Path $verification ('workspace-startup-'+[Guid]::NewGuid().ToString())
if (-not [IO.Path]::GetFullPath($sandbox).StartsWith($verification+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {throw 'Unexpected sandbox directory.'}
$store=Join-Path $sandbox 'rust-data'
$null=New-Item -ItemType Directory -Path $store
$native=Join-Path $sandbox 'yamantaka-rust.exe'
Copy-Item -LiteralPath $executablePath -Destination $native
$null=New-Item -ItemType File -Path (Join-Path $sandbox '.yamantaka-rust-portable')
$helper=Join-Path $sandbox 'prepare.py'
@'
import hashlib, json, sqlite3, sys
from pathlib import Path
source, root, mode = Path(sys.argv[1]).resolve(strict=True), Path(sys.argv[2]).resolve(strict=True), sys.argv[3]
assert root.name == 'rust-data' and root.parent.name.startswith('workspace-startup-')
database = root / 'yamantaka.sqlite3'
if mode == 'prepare':
    with sqlite3.connect((source / 'export-preview.sqlite3').as_uri() + '?mode=ro', uri=True) as original:
        with sqlite3.connect(database) as target:
            original.backup(target)
    (root / 'yamantaka.reader-continuity.json').write_bytes((source / 'export-preview.reader-continuity.json').read_bytes())
    (root / 'preferences.json').write_text(json.dumps({'version': 1, 'legacy': {}, 'rust': {}}), encoding='utf-8')
with sqlite3.connect(database.as_uri() + '?mode=ro', uri=True) as db:
    content = [db.execute('SELECT * FROM workspaces ORDER BY id').fetchall(), db.execute('SELECT * FROM workspace_tabs ORDER BY id').fetchall()]
digest = hashlib.sha256(json.dumps(content, ensure_ascii=False).encode()).hexdigest()
baseline = root.parent / 'workspace-content.sha256'
if mode == 'prepare':
    baseline.write_text(digest, encoding='ascii')
else:
    assert digest == baseline.read_text(encoding='ascii'), 'Startup rewrote saved workspace data'
if mode == 'default':
    continuity = root / 'yamantaka.reader-continuity.json'
    assert continuity.is_file() and not continuity.is_symlink()
    continuity.unlink()
'@ | Set-Content -LiteralPath $helper -Encoding UTF8
& $Python $helper $source $store 'prepare'
if ($LASTEXITCODE -ne 0) {throw 'Could not prepare the synthetic portable store.'}
$results=@()
try {
    foreach ($case in @('bound-continuity','default-without-continuity')) {
        if ($case -eq 'default-without-continuity') {
            & $Python $helper $source $store 'default'
            if ($LASTEXITCODE -ne 0) {throw 'Could not prepare default recovery.'}
        }
        $image=Join-Path $sandbox ($case+'.png')
        $process=Start-Process -FilePath $native -ArgumentList @(('"--capture={0}"' -f $image),'--es') -WorkingDirectory $repository -WindowStyle Hidden -RedirectStandardError (Join-Path $sandbox ($case+'.stderr.log')) -PassThru
        Wait-RustTestProcess -Process $process
        if ($process.ExitCode -ne 0) {throw "Startup failed: $case"}
        $actual=Get-Content -LiteralPath ([IO.Path]::ChangeExtension($image,'controls.json')) -Raw -Encoding UTF8 | ConvertFrom-Json
        if ($actual.synthetic -or -not $actual.libraryQueryLoaded -or -not $actual.workspaces.startupFinished -or $actual.workspaces.bindingName -ne 'Reference copy' -or $actual.workspaces.paused -or -not $actual.workspaces.focus -or $actual.workspaces.reference -ne $actual.workspaces.bindingId -or @($actual.workspaces.queue).Count -ne 3) {throw "Incorrect workspace recovery: $case"}
        if (@($actual.workspaces.queue | Where-Object {$_.status -eq 'reviewed' -and $_.nextAction -eq 'Study key figures'}).Count -ne 1) {throw "Queue metadata changed: $case"}
        if (@($actual.workspaces.queue | Where-Object {-not $_.title}).Count -ne 0) {throw "Reading titles were not recovered: $case"}
        & $Python $helper $source $store 'verify'
        if ($LASTEXITCODE -ne 0) {throw "Saved workspace data changed: $case"}
        $results += [ordered]@{case=$case;productionStartup=$true;binding=$actual.workspaces.bindingName;queueEntries=@($actual.workspaces.queue).Count;focus=$actual.workspaces.focus}
        Write-Output "Workspace startup verified: $case"
    }
    [ordered]@{version=1;syntheticFixture=$true;executableSha256=(Get-FileHash -LiteralPath $executablePath).Hash;cases=$results;scope='Two isolated production portable startups with native PDF opening. No personal store or physical interaction acceptance.'} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $sandbox 'result.json') -Encoding UTF8
    Write-Output "Result: $(Join-Path $sandbox 'result.json')"
} finally {
    if ((Test-Path -LiteralPath $native) -and (Get-FileHash -LiteralPath $native).Hash -eq (Get-FileHash -LiteralPath $executablePath).Hash) {Remove-Item -LiteralPath $native}
}
