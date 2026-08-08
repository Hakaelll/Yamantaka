function Get-YamantakaWindowsNativeBinaries {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath
    )

    $resolvedBundle = [System.IO.Path]::GetFullPath($BundlePath)
    if (-not (Test-Path -LiteralPath $resolvedBundle -PathType Container)) {
        throw "Windows bundle directory not found: $resolvedBundle"
    }

    $binaries = @(
        Get-ChildItem -LiteralPath $resolvedBundle -File -Recurse |
            Where-Object { $_.Extension.ToLowerInvariant() -in @('.exe', '.dll') } |
            Sort-Object FullName
    )
    if ($binaries.Count -eq 0) {
        throw "No native .exe or .dll files were found in: $resolvedBundle"
    }
    return $binaries
}

function Get-YamantakaWindowsBundleLayoutState {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath
    )

    $resolvedBundle = [System.IO.Path]::GetFullPath($BundlePath).TrimEnd('\')
    if (-not (Test-Path -LiteralPath $resolvedBundle -PathType Container)) {
        throw "Windows bundle directory not found: $resolvedBundle"
    }

    $requiredPaths = @(
        'yamantaka.exe',
        'flutter_windows.dll',
        'data\app.so',
        'data\icudtl.dat'
    )
    $missingPaths = @(
        $requiredPaths |
            Where-Object {
                -not (Test-Path -LiteralPath (Join-Path $resolvedBundle $_) -PathType Leaf)
            }
    )

    return [pscustomobject]@{
        bundlePath = $resolvedBundle
        ready = $missingPaths.Count -eq 0
        requiredPaths = $requiredPaths
        missingPaths = $missingPaths
    }
}

function Get-YamantakaWindowsBundleExecutionState {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath,
        [Parameter(Mandatory = $true)][string]$RepositoryRoot
    )

    $resolvedRepository = [System.IO.Path]::GetFullPath($RepositoryRoot).TrimEnd('\')
    $layout = Get-YamantakaWindowsBundleLayoutState -BundlePath $BundlePath
    $signatureState = Get-YamantakaWindowsBundleSignatureState `
        -BundlePath $layout.bundlePath
    $buildRoot = [System.IO.Path]::GetFullPath(
        (Join-Path $resolvedRepository 'build')
    ).TrimEnd('\')
    $buildPrefix = $buildRoot + '\'
    $isRepositoryBuild = $layout.bundlePath.StartsWith(
        $buildPrefix,
        [System.StringComparison]::OrdinalIgnoreCase
    )

    $markedBinaries = @(
        Get-YamantakaWindowsNativeBinaries -BundlePath $layout.bundlePath |
            ForEach-Object {
                $zone = Get-Item -LiteralPath $_.FullName -Stream Zone.Identifier `
                    -ErrorAction SilentlyContinue
                if ($zone) {
                    $_.FullName.Substring($layout.bundlePath.Length + 1).Replace('\', '/')
                }
            }
    )
    $unsignedLocalBuildReady = (
        $layout.ready -and
        $isRepositoryBuild -and
        $markedBinaries.Count -eq 0
    )
    $readyForLocalExecution = (
        $layout.ready -and (
            $signatureState.readyForNativeExecution -or
            $unsignedLocalBuildReady
        )
    )

    $mode = if ($layout.ready -and $signatureState.readyForNativeExecution) {
        'signed'
    }
    elseif ($unsignedLocalBuildReady) {
        'unsigned-local-build'
    }
    else {
        'blocked'
    }
    $reason = if (-not $layout.ready) {
        "Bundle is incomplete; missing: $($layout.missingPaths -join ', ')"
    }
    elseif (-not $isRepositoryBuild) {
        'Unsigned execution is limited to the repository build directory.'
    }
    elseif ($markedBinaries.Count -gt 0) {
        "One or more native binaries carry Mark-of-the-Web: $($markedBinaries -join ', ')"
    }
    elseif ($signatureState.readyForNativeExecution) {
        'Every native binary has a locally valid Authenticode signature.'
    }
    else {
        'The complete bundle is a local repository build without Mark-of-the-Web.'
    }

    return [pscustomobject]@{
        bundlePath = $layout.bundlePath
        mode = $mode
        reason = $reason
        readyForLocalExecution = $readyForLocalExecution
        unsignedLocalBuildReady = $unsignedLocalBuildReady
        isRepositoryBuild = $isRepositoryBuild
        markedBinaries = $markedBinaries
        layoutState = $layout
        signatureState = $signatureState
    }
}

function Get-YamantakaWindowsBundleSignatureState {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath
    )

    $resolvedBundle = [System.IO.Path]::GetFullPath($BundlePath).TrimEnd('\')
    $prefix = $resolvedBundle + '\'
    $entries = @(
        Get-YamantakaWindowsNativeBinaries -BundlePath $resolvedBundle |
            ForEach-Object {
                $signature = Get-AuthenticodeSignature -LiteralPath $_.FullName
                $relativePath = if ($_.FullName.StartsWith(
                        $prefix,
                        [System.StringComparison]::OrdinalIgnoreCase
                    )) {
                    $_.FullName.Substring($prefix.Length).Replace('\', '/')
                }
                else {
                    $_.Name
                }
                [pscustomobject]@{
                    relativePath = $relativePath
                    status = [string]$signature.Status
                    valid = $signature.Status -eq [System.Management.Automation.SignatureStatus]::Valid
                    subject = if ($signature.SignerCertificate) {
                        $signature.SignerCertificate.Subject
                    }
                    else {
                        $null
                    }
                    thumbprint = if ($signature.SignerCertificate) {
                        $signature.SignerCertificate.Thumbprint
                    }
                    else {
                        $null
                    }
                }
            }
    )
    $invalid = @($entries | Where-Object { -not $_.valid })
    return [pscustomobject]@{
        bundlePath = $resolvedBundle
        readyForNativeExecution = $entries.Count -gt 0 -and $invalid.Count -eq 0
        readyForSignedDistribution = $entries.Count -gt 0 -and $invalid.Count -eq 0
        binaryCount = $entries.Count
        validSignatureCount = $entries.Count - $invalid.Count
        invalidOrUnsignedCount = $invalid.Count
        binaries = $entries
    }
}

function Assert-YamantakaWindowsBundleSigned {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath
    )

    $state = Get-YamantakaWindowsBundleSignatureState -BundlePath $BundlePath
    if (-not $state.readyForNativeExecution) {
        $problems = @(
            $state.binaries |
                Where-Object { -not $_.valid } |
                ForEach-Object { "$($_.relativePath) [$($_.status)]" }
        ) -join ', '
        throw @"
Native execution was refused before Start-Process because the bundle does not
have valid Authenticode signatures on every .exe and .dll: $problems

This gate requires a distribution-ready bundle. Sign it with
tool/sign_windows_bundle.ps1 and an RSA code-signing certificate from a trusted
provider, then retry. For a source-built bundle under this repository, use a
local-execution gate instead. Do not disable Windows security protections.
"@
    }
    return $state
}

function Assert-YamantakaWindowsBundleRunnable {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$BundlePath,
        [Parameter(Mandatory = $true)][string]$RepositoryRoot,
        [switch]$RequireSignature
    )

    if ($RequireSignature) {
        [void](Assert-YamantakaWindowsBundleSigned -BundlePath $BundlePath)
    }

    $state = Get-YamantakaWindowsBundleExecutionState `
        -BundlePath $BundlePath `
        -RepositoryRoot $RepositoryRoot
    if (-not $state.readyForLocalExecution) {
        throw @"
Native execution was refused before Start-Process: $($state.reason)

Keep the complete Flutter bundle together. Only unsigned builds located under
this repository's build directory and without Mark-of-the-Web are accepted for
local testing. Sign every .exe and .dll before distribution.
"@
    }
    if ($state.mode -eq 'unsigned-local-build') {
        Write-Warning @"
Running an unsigned local build from this repository. This is suitable for
local testing only; it is not a signed distribution artifact.
"@
    }
    return $state
}

function Resolve-YamantakaSignTool {
    [CmdletBinding()]
    param(
        [string]$SignToolPath
    )

    if ($SignToolPath) {
        $resolved = [System.IO.Path]::GetFullPath($SignToolPath)
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
            throw "SignTool was not found: $resolved"
        }
        return $resolved
    }

    $command = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }

    $sdkRoots = [System.Collections.Generic.List[string]]::new()
    $programFilesX86 = [System.Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
    if ($programFilesX86) {
        $sdkRoots.Add((Join-Path $programFilesX86 'Windows Kits\10\bin'))
    }
    $programFiles = [System.Environment]::GetEnvironmentVariable('ProgramFiles')
    if ($programFiles) {
        $sdkRoots.Add((Join-Path $programFiles 'Windows Kits\10\bin'))
    }

    $candidate = @(
        $sdkRoots |
            Where-Object { Test-Path -LiteralPath $_ -PathType Container } |
            ForEach-Object {
                Get-ChildItem -LiteralPath $_ -Filter signtool.exe -File -Recurse |
                    Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' }
            } |
            Sort-Object FullName -Descending
    ) | Select-Object -First 1
    if (-not $candidate) {
        throw 'signtool.exe was not found on PATH or in the Windows 10/11 SDK.'
    }
    return $candidate.FullName
}
