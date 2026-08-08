[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')

function Assert-Contract {
    param(
        [Parameter(Mandatory = $true)][bool]$Condition,
        [Parameter(Mandatory = $true)][string]$Message
    )
    if (-not $Condition) { throw $Message }
}

$fixtureRoot = Join-Path $repositoryRoot (
    'build/verification/windows-binary-trust-' + [Guid]::NewGuid().ToString('N')
)
$buildRoot = [System.IO.Path]::GetFullPath(
    (Join-Path $repositoryRoot 'build')
).TrimEnd('\') + '\'
if (-not $fixtureRoot.StartsWith(
        $buildRoot,
        [System.StringComparison]::OrdinalIgnoreCase
    )) {
    throw "Refusing to create a trust fixture outside the build directory: $fixtureRoot"
}

try {
    New-Item -ItemType Directory -Force -Path (Join-Path $fixtureRoot 'data') |
        Out-Null
    $peStub = [byte[]](0x4d, 0x5a, 0x00, 0x00)
    [System.IO.File]::WriteAllBytes(
        (Join-Path $fixtureRoot 'yamantaka.exe'),
        $peStub
    )
    [System.IO.File]::WriteAllBytes(
        (Join-Path $fixtureRoot 'flutter_windows.dll'),
        $peStub
    )
    [System.IO.File]::WriteAllBytes(
        (Join-Path $fixtureRoot 'data/app.so'),
        [byte[]](0x01)
    )
    [System.IO.File]::WriteAllBytes(
        (Join-Path $fixtureRoot 'data/icudtl.dat'),
        [byte[]](0x01)
    )

    $state = Get-YamantakaWindowsBundleExecutionState `
        -BundlePath $fixtureRoot `
        -RepositoryRoot $repositoryRoot
    Assert-Contract $state.readyForLocalExecution `
        'A complete unsigned repository build should be accepted for local execution.'
    Assert-Contract ($state.mode -eq 'unsigned-local-build') `
        "Unexpected local execution mode: $($state.mode)"
    Assert-Contract (-not $state.signatureState.readyForNativeExecution) `
        'The synthetic unsigned bundle must not be considered signed.'

    [void](Assert-YamantakaWindowsBundleRunnable `
            -BundlePath $fixtureRoot `
            -RepositoryRoot $repositoryRoot)

    $signatureWasRequired = $false
    try {
        [void](Assert-YamantakaWindowsBundleRunnable `
                -BundlePath $fixtureRoot `
                -RepositoryRoot $repositoryRoot `
                -RequireSignature)
    }
    catch {
        $signatureWasRequired = $true
    }
    Assert-Contract $signatureWasRequired `
        '-RequireSignature must reject an unsigned bundle.'

    $foreignRoot = Join-Path $repositoryRoot 'not-the-owning-repository'
    $outsideRepository = Get-YamantakaWindowsBundleExecutionState `
        -BundlePath $fixtureRoot `
        -RepositoryRoot $foreignRoot
    Assert-Contract (-not $outsideRepository.readyForLocalExecution) `
        'An unsigned bundle outside the owning repository build root must be rejected.'

    $fixtureExecutable = Join-Path $fixtureRoot 'yamantaka.exe'
    Set-Content -LiteralPath $fixtureExecutable -Stream Zone.Identifier `
        -Value "[ZoneTransfer]`nZoneId=3"
    $marked = Get-YamantakaWindowsBundleExecutionState `
        -BundlePath $fixtureRoot `
        -RepositoryRoot $repositoryRoot
    Assert-Contract (-not $marked.readyForLocalExecution) `
        'A bundle carrying Mark-of-the-Web must not pass the local gate.'
    Remove-Item -LiteralPath $fixtureExecutable -Stream Zone.Identifier

    [System.IO.File]::Delete((Join-Path $fixtureRoot 'data/icudtl.dat'))
    $incomplete = Get-YamantakaWindowsBundleExecutionState `
        -BundlePath $fixtureRoot `
        -RepositoryRoot $repositoryRoot
    Assert-Contract (-not $incomplete.readyForLocalExecution) `
        'An incomplete Flutter bundle must not be executable through the local gate.'

    Write-Host 'Windows binary trust contracts passed.'
}
finally {
    if (Test-Path -LiteralPath $fixtureRoot) {
        [System.IO.Directory]::Delete($fixtureRoot, $true)
    }
}
