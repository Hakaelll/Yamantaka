[CmdletBinding()]
param(
    [switch]$AllowDirty,
    [switch]$SkipNative,
    [switch]$SkipVisual,
    [switch]$SkipCorpus,
    [switch]$SkipTests,
    [switch]$SkipPerformance,
    [switch]$SkipToolchain,
    [switch]$SkipDependencies,
    [switch]$SkipFormat,
    [switch]$SkipAnalysis,
    [string]$FlutterExecutable,
    [string]$DartExecutable,
    [string]$CertificateThumbprint,
    [string]$SignToolPath,
    [string]$TimestampUrl = 'http://timestamp.digicert.com',
    [switch]$SkipTimestamp,
    [switch]$MachineCertificateStore,
    [switch]$RequireSignature,
    [switch]$SkipDpiMatrix,
    [switch]$DpiScaleConfirmed,
    [ValidateRange(1, 3600)]
    [int]$ExternalTimeoutSeconds = 180,
    [string]$OutputDirectory = 'build/verification'
)

$ErrorActionPreference = 'Stop'
$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'windows_binary_trust.ps1')
$outputRoot = [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $OutputDirectory))
$failures = [System.Collections.Generic.List[string]]::new()
$phaseResults = [System.Collections.Generic.List[object]]::new()
$startedAt = [DateTime]::UtcNow
$nativeExecutionReady = $false
$nativeExecutionReason = 'Windows release build has not been evaluated.'
$nativeSignatureState = $null
$nativeExecutionState = $null
$resolvedPubCache = $env:PUB_CACHE
if ([string]::IsNullOrWhiteSpace($resolvedPubCache)) {
    $defaultPubCache = Join-Path $env:LOCALAPPDATA 'Pub\Cache'
    if (Test-Path -LiteralPath $defaultPubCache) {
        $resolvedPubCache = $defaultPubCache
    }
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
$toolStateRoot = Join-Path $outputRoot 'tool-state'
New-Item -ItemType Directory -Force -Path $toolStateRoot | Out-Null
Push-Location $repositoryRoot
try {
    $env:APPDATA = $toolStateRoot
    $env:LOCALAPPDATA = $toolStateRoot
    # The verification gate must remain offline. Suppress Flutter's optional
    # analytics before invoking any tool command so a successful phase cannot
    # be turned into a network-related failure while reporting telemetry.
    $env:FLUTTER_SUPPRESS_ANALYTICS = 'true'
    $env:DART_SUPPRESS_ANALYTICS = 'true'
    if (-not [string]::IsNullOrWhiteSpace($resolvedPubCache)) {
        $env:PUB_CACHE = $resolvedPubCache
    }
    $env:GIT_CONFIG_COUNT = '2'
    $env:GIT_CONFIG_KEY_0 = 'safe.directory'
    $env:GIT_CONFIG_VALUE_0 = $repositoryRoot.Replace('\', '/')
    $env:GIT_CONFIG_KEY_1 = 'safe.directory'
    $env:GIT_CONFIG_VALUE_1 = (Join-Path $repositoryRoot '.flutter-sdk').Replace('\', '/')
    # Codex can inherit both PATH and Path. Start-Process treats environment
    # keys case-insensitively and otherwise refuses to launch any child.
    $processPath = [System.Environment]::GetEnvironmentVariable('Path', 'Process')
    [System.Environment]::SetEnvironmentVariable('PATH', $null, 'Process')
    [System.Environment]::SetEnvironmentVariable('Path', $processPath, 'Process')

    function Resolve-Executable {
        param(
            [string]$Requested,
            [string]$LocalPath,
            [string]$Name
        )
        if ($Requested) { return [System.IO.Path]::GetFullPath($Requested) }
        if (Test-Path $LocalPath) { return [System.IO.Path]::GetFullPath($LocalPath) }
        $command = Get-Command $Name -ErrorAction SilentlyContinue
        if ($command) { return $command.Source }
        throw "$Name was not found on PATH or in .flutter-sdk."
    }

    function Invoke-Gate {
        param(
            [Parameter(Mandatory = $true)][string]$Name,
            [Parameter(Mandatory = $true)][scriptblock]$Action
        )
        Write-Host "`n=== $Name ===" -ForegroundColor Cyan
        try {
            & $Action
            $phaseResults.Add([ordered]@{
                    name = $Name
                    status = 'passed'
                })
            Write-Host "PASS $Name" -ForegroundColor Green
        }
        catch {
            $message = "${Name}: $($_.Exception.Message)"
            $failures.Add($message)
            $phaseResults.Add([ordered]@{
                    name = $Name
                    status = 'failed'
                    message = $_.Exception.Message
                })
            Write-Host "FAIL $message" -ForegroundColor Red
        }
    }

    function Add-SkippedGate {
        param(
            [Parameter(Mandatory = $true)][string]$Name,
            [Parameter(Mandatory = $true)][string]$Reason
        )
        $phaseResults.Add([ordered]@{
                name = $Name
                status = 'skipped'
                reason = $Reason
            })
        Write-Host "SKIP $Name ($Reason)" -ForegroundColor Yellow
    }

    function Invoke-External {
        param(
            [Parameter(Mandatory = $true)][string]$Executable,
            [Parameter(Mandatory = $true)][string[]]$Arguments,
            [string]$LogName
        )
        if ([string]::IsNullOrWhiteSpace($Executable)) {
            throw "Executable for '$LogName' was not resolved."
        }
        $logPath = if ($LogName) { Join-Path $outputRoot $LogName } else { $null }
        $processExecutable = $Executable
        $processArguments = $Arguments
        if ($Executable -match '\.(bat|cmd)$') {
            $quotedArguments = @($Arguments | ForEach-Object {
                    if ($_ -match '[\s"]') {
                        '"' + $_.Replace('"', '\"') + '"'
                    }
                    else {
                        $_
                    }
                })
            $batchCommand = '"' + $Executable + '"'
            if ($quotedArguments.Count -gt 0) {
                $batchCommand += ' ' + ($quotedArguments -join ' ')
            }
            $processExecutable = $env:ComSpec
            $processArguments = '/d /s /c "' + $batchCommand + '"'
        }
        if ($processArguments -is [array]) {
            $processArgumentLine = ($processArguments | ForEach-Object {
                    if ($_ -match '[\s"]') {
                        '"' + $_.Replace('"', '\"') + '"'
                    }
                    else {
                        $_
                    }
                }) -join ' '
        }
        else {
            $processArgumentLine = [string]$processArguments
        }
        $startInfo = New-Object System.Diagnostics.ProcessStartInfo
        $startInfo.FileName = $processExecutable
        $startInfo.Arguments = $processArgumentLine
        $startInfo.WorkingDirectory = $repositoryRoot
        $startInfo.UseShellExecute = $false
        $startInfo.CreateNoWindow = $true
        $startInfo.RedirectStandardOutput = $true
        $startInfo.RedirectStandardError = $true
        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $startInfo
        if (-not $process.Start()) {
            throw "Could not start $processExecutable."
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $completed = $process.WaitForExit($ExternalTimeoutSeconds * 1000)
        if (-not $completed) {
            try {
                $process.Kill()
            }
            catch {
                # The process may have exited between the timeout and Kill().
            }
            $process.WaitForExit()
            $timeoutStdout = $stdoutTask.Result
            $timeoutStderr = $stderrTask.Result
            if ($logPath) {
                Set-Content -LiteralPath $logPath -Value (@($timeoutStdout, $timeoutStderr) -join '') -Encoding UTF8
            }
            if ($timeoutStdout) { Write-Host $timeoutStdout.TrimEnd() }
            if ($timeoutStderr) { Write-Host $timeoutStderr.TrimEnd() -ForegroundColor DarkYellow }
            throw "$Executable exceeded the ${ExternalTimeoutSeconds}s timeout."
        }
        $process.WaitForExit()
        $stdout = $stdoutTask.Result
        $stderr = $stderrTask.Result
        if ($logPath) {
            Set-Content -LiteralPath $logPath -Value (@($stdout, $stderr) -join '') -Encoding UTF8
        }
        if ($stdout) { Write-Host $stdout.TrimEnd() }
        if ($stderr) { Write-Host $stderr.TrimEnd() -ForegroundColor DarkYellow }
        if ($process.ExitCode -ne 0) {
            throw "$Executable returned exit code $($process.ExitCode)."
        }
    }

    $toolchainResolutionError = $null
    try {
        $FlutterExecutable = Resolve-Executable `
            -Requested $FlutterExecutable `
            -LocalPath (Join-Path $repositoryRoot '.flutter-sdk/bin/flutter.bat') `
            -Name 'flutter'
        $DartExecutable = Resolve-Executable `
            -Requested $DartExecutable `
            -LocalPath (Join-Path $repositoryRoot '.flutter-sdk/bin/dart.bat') `
            -Name 'dart'
    }
    catch {
        $toolchainResolutionError = $_.Exception.Message
    }

    Invoke-Gate 'repository hygiene' {
        $status = @(git status --porcelain --untracked-files=all)
        if (-not $AllowDirty -and $status.Count -gt 0) {
            throw 'The checkout is dirty. Pass -AllowDirty only when the diff is intentional.'
        }
        if (-not (Test-Path 'third_party/pdfrx/PATCHES.md')) {
            throw 'The pdfrx patch manifest is missing.'
        }
        git diff --check
        if ($LASTEXITCODE -ne 0) { throw 'Whitespace errors were found by git diff --check.' }
    }

    if (-not $SkipToolchain) {
        Invoke-Gate 'toolchain and native prerequisites' {
            if ($toolchainResolutionError) { throw $toolchainResolutionError }
            if (-not $SkipNative) {
                # Flutter resolves CMake and MSBuild from the selected Visual
                # Studio installation even when neither executable is on PATH.
                # The native release gate below is the authoritative check.
                $driveRoot = [System.IO.Path]::GetPathRoot($repositoryRoot)
                $free = ([System.IO.DriveInfo]::new($driveRoot)).AvailableFreeSpace
                if ($free -lt 10GB) {
                    throw "Less than 10 GB is free on the workspace drive ($free bytes)."
                }
            }
            Invoke-External $FlutterExecutable @('doctor', '-v') 'flutter-doctor.log'
        }
    }
    else {
        Add-SkippedGate 'toolchain and native prerequisites' 'SkipToolchain'
    }

    if (-not $SkipDependencies) {
        Invoke-Gate 'dependency resolution' {
            Invoke-External $FlutterExecutable @('pub', 'get', '--offline') 'flutter-pub-get.log'
        }
    }
    else {
        Add-SkippedGate 'dependency resolution' 'SkipDependencies'
    }

    Invoke-Gate 'pure contract self-checks' {
        Invoke-External $DartExecutable @('--enable-asserts', 'run', 'tool/self_check.dart') 'self-check.log'
        & (Join-Path $PSScriptRoot 'test_windows_binary_trust.ps1')
    }

    if (-not $SkipFormat) {
        Invoke-Gate 'format check' {
            Invoke-External $DartExecutable @('format', '--output=none', '--set-exit-if-changed', 'lib', 'test', 'tool') 'dart-format.log'
        }
    }
    else {
        Add-SkippedGate 'format check' 'SkipFormat'
    }

    if (-not $SkipAnalysis) {
        Invoke-Gate 'static analysis' {
            Invoke-External $FlutterExecutable @('analyze', '--no-pub') 'flutter-analyze.log'
        }
    }
    else {
        Add-SkippedGate 'static analysis' 'SkipAnalysis'
    }

    if (-not $SkipTests) {
        Invoke-Gate 'unit and widget tests' {
            Invoke-External $FlutterExecutable @('test', '--no-pub', '--coverage') 'flutter-test.log'
        }
    }
    else {
        Add-SkippedGate 'unit and widget tests' 'SkipTests'
    }

    if (-not $SkipCorpus) {
        Invoke-Gate 'synthetic pathological PDF corpus' {
            $corpus = Join-Path $outputRoot 'corpus'
            Invoke-External $DartExecutable @('run', 'tool/create_pathological_corpus.dart', $corpus) 'corpus.log'
            $manifestPath = Join-Path $corpus 'manifest.json'
            if (-not (Test-Path $manifestPath)) {
                throw 'The corpus generator did not produce a manifest.'
            }
            $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
            if ($manifest.containsPersonalData -ne $false -or $manifest.schemaVersion -lt 1) {
                throw 'The corpus manifest is not marked as synthetic and private.'
            }
        }
    }
    else {
        Add-SkippedGate 'synthetic pathological PDF corpus' 'SkipCorpus'
    }

    if (-not $SkipPerformance) {
        Invoke-Gate 'library performance benchmark' {
            $performancePath = Join-Path $outputRoot 'performance/library-benchmark.json'
            Invoke-External $DartExecutable @('run', 'tool/benchmark_library.dart', $performancePath) 'library-benchmark.log'
            $performance = Get-Content $performancePath -Raw | ConvertFrom-Json
            $results = @($performance.results)
            $fiveThousand = @(
                $results | Where-Object { $_.documents -eq 5000 }
            ) | Select-Object -First 1
            $invalid = @(
                $results | Where-Object {
                    $_.integrity -ne 'ok' -or
                    $_.materializedDocuments -gt $performance.pageSize -or
                    $_.totalDocuments -ne $_.documents
                }
            )
            if (
                $performance.containsPersonalData -ne $false -or
                $results.Count -eq 0 -or
                -not $fiveThousand -or
                $invalid.Count -gt 0
            ) {
                throw 'The performance fixture did not complete with intact, bounded databases including 5,000 documents.'
            }
        }
    }
    else {
        Add-SkippedGate '5,000-document performance benchmark' 'SkipPerformance'
    }

    if (-not $SkipNative) {
        Invoke-Gate 'Windows release build' {
            Invoke-External $FlutterExecutable @('build', 'windows', '--release', '--no-pub') 'flutter-build-windows.log'
        }

        $releaseBuildPassed = @(
            $phaseResults |
                Where-Object { $_.name -eq 'Windows release build' } |
                Select-Object -Last 1
        ).status -eq 'passed'
        if ($releaseBuildPassed -and $CertificateThumbprint) {
            Invoke-Gate 'Windows bundle signing' {
                $signingArguments = @{
                    BundlePath = 'build/windows/x64/runner/Release'
                    CertificateThumbprint = $CertificateThumbprint
                    TimestampUrl = $TimestampUrl
                }
                if ($SignToolPath) {
                    $signingArguments.SignToolPath = $SignToolPath
                }
                if ($SkipTimestamp) {
                    $signingArguments.SkipTimestamp = $true
                }
                if ($MachineCertificateStore) {
                    $signingArguments.MachineCertificateStore = $true
                }
                & (Join-Path $PSScriptRoot 'sign_windows_bundle.ps1') @signingArguments
                if ($LASTEXITCODE -ne 0) {
                    throw "Windows bundle signing returned exit code $LASTEXITCODE."
                }
            }
        }

        $signingPassed = -not $CertificateThumbprint -or @(
            $phaseResults |
                Where-Object { $_.name -eq 'Windows bundle signing' } |
                Select-Object -Last 1
        ).status -eq 'passed'
        if ($releaseBuildPassed -and $signingPassed) {
            try {
                $nativeExecutionState = Get-YamantakaWindowsBundleExecutionState `
                    -BundlePath (Join-Path $repositoryRoot 'build/windows/x64/runner/Release') `
                    -RepositoryRoot $repositoryRoot
                $nativeSignatureState = $nativeExecutionState.signatureState
                if ($RequireSignature) {
                    $nativeExecutionReady = $nativeSignatureState.readyForNativeExecution
                    if ($nativeExecutionReady) {
                        $nativeExecutionReason = 'Every native binary has a locally valid Authenticode signature.'
                    }
                    else {
                        $nativeExecutionReason = "Automatic launch suppressed because -RequireSignature was set: $($nativeSignatureState.invalidOrUnsignedCount) of $($nativeSignatureState.binaryCount) native binaries are unsigned or invalid."
                    }
                }
                else {
                    $nativeExecutionReady = $nativeExecutionState.readyForLocalExecution
                    $nativeExecutionReason = $nativeExecutionState.reason
                }
            }
            catch {
                $nativeExecutionReason = "Automatic launch suppressed: $($_.Exception.Message)"
            }
        }
        elseif (-not $releaseBuildPassed) {
            $nativeExecutionReason = 'Automatic launch suppressed because the Windows release build failed.'
        }
        else {
            $nativeExecutionReason = 'Automatic launch suppressed because Windows bundle signing failed.'
        }

        if (-not $SkipCorpus -and $nativeExecutionReady) {
            Invoke-Gate 'zero-network smoke' {
                & (Join-Path $PSScriptRoot 'verify_no_network.ps1') `
                    -ExecutablePath 'build/windows/x64/runner/Release/yamantaka.exe' `
                    -FixturePath (Join-Path $outputRoot 'corpus/normal.pdf') `
                    -OutputPath (Join-Path $OutputDirectory 'no-network.json') `
                    -RequireSignature:$RequireSignature
                if ($LASTEXITCODE -ne 0) {
                    throw "Zero-network smoke returned exit code $LASTEXITCODE."
                }
            }
        }
        elseif (-not $SkipCorpus) {
            Add-SkippedGate 'zero-network smoke' $nativeExecutionReason
        }
        else {
            Add-SkippedGate 'zero-network smoke' 'SkipCorpus'
        }
    }
    else {
        Add-SkippedGate 'Windows release build' 'SkipNative'
        Add-SkippedGate 'zero-network smoke' 'SkipNative'
    }

    if (-not $SkipVisual) {
        Invoke-Gate 'maintained reader visual baseline' {
            Invoke-External $FlutterExecutable @(
                'test',
                '--no-pub',
                'test/reader_widgets_test.dart',
                '--plain-name',
                'book reader matches the maintained editorial v0.10 baseline'
            ) 'reader-visual-baseline.log'
        }
        Invoke-Gate 'maintained document navigator visual baseline' {
            Invoke-External $FlutterExecutable @(
                'test',
                '--no-pub',
                'test/reader_widgets_test.dart',
                '--plain-name',
                'document navigator matches the maintained v0.11 baseline'
            ) 'document-navigator-visual-baseline.log'
        }
    }
    else {
        Add-SkippedGate 'maintained reader visual baseline' 'SkipVisual'
        Add-SkippedGate 'maintained document navigator visual baseline' 'SkipVisual'
    }

    if (-not $SkipDpiMatrix -and -not $SkipNative -and $nativeExecutionReady) {
        Invoke-Gate 'native resolution and DPI matrix' {
            $matrixScript = Join-Path $PSScriptRoot 'run_visual_matrix.ps1'
            $matrixArguments = @{
                ExecutablePath = 'build/windows/x64/runner/Release/yamantaka.exe'
                FixturePdf = Join-Path $OutputDirectory 'corpus/normal.pdf'
                OutputDirectory = Join-Path $OutputDirectory 'dpi-matrix'
                ScaleConfirmed = [bool]$DpiScaleConfirmed
                RequireSignature = [bool]$RequireSignature
            }
            & $matrixScript @matrixArguments
            if ($LASTEXITCODE -ne 0) {
                throw "DPI matrix returned exit code $LASTEXITCODE."
            }
        }
    }
    else {
        $dpiSkipReason = if ($SkipDpiMatrix) {
            'SkipDpiMatrix'
        }
        elseif ($SkipNative) {
            'SkipNative'
        }
        else {
            $nativeExecutionReason
        }
        Add-SkippedGate 'native resolution and DPI matrix' $dpiSkipReason
    }

    $skippedCount = @($phaseResults | Where-Object { $_.status -eq 'skipped' }).Count
    $reportStatus = if ($failures.Count -gt 0) { 'failed' } elseif ($skippedCount -gt 0) { 'partial' } else { 'passed' }
    $report = [ordered]@{
        format = 'yamantaka-windows-verification'
        version = 1
        status = $reportStatus
        startedAt = $startedAt.ToString('o')
        finishedAt = [DateTime]::UtcNow.ToString('o')
        repository = $repositoryRoot
        allowDirty = [bool]$AllowDirty
        skipNative = [bool]$SkipNative
        skipVisual = [bool]$SkipVisual
        skipCorpus = [bool]$SkipCorpus
        skipTests = [bool]$SkipTests
        skipPerformance = [bool]$SkipPerformance
        skipToolchain = [bool]$SkipToolchain
        skipDependencies = [bool]$SkipDependencies
        skipFormat = [bool]$SkipFormat
        skipAnalysis = [bool]$SkipAnalysis
        skipDpiMatrix = [bool]$SkipDpiMatrix
        dpiScaleConfirmed = [bool]$DpiScaleConfirmed
        certificateThumbprintProvided = -not [string]::IsNullOrWhiteSpace($CertificateThumbprint)
        requireSignature = [bool]$RequireSignature
        nativeExecutionReady = $nativeExecutionReady
        nativeExecutionReason = $nativeExecutionReason
        nativeExecutionState = $nativeExecutionState
        nativeSignatureState = $nativeSignatureState
        externalTimeoutSeconds = $ExternalTimeoutSeconds
        flutterExecutable = $FlutterExecutable
        dartExecutable = $DartExecutable
        toolStateDirectory = $toolStateRoot
        phases = @($phaseResults)
        failures = @($failures)
        skippedPhaseCount = $skippedCount
    }
    $report | ConvertTo-Json -Depth 8 | Set-Content -Encoding UTF8 (Join-Path $outputRoot 'verification-report.json')
}
finally {
    Pop-Location
}

if ($failures.Count -gt 0) {
    Write-Host "`n$($failures.Count) gate(s) failed. See $outputRoot." -ForegroundColor Red
    exit 1
}
if ($skippedCount -gt 0) {
    Write-Host "`nRequested gates passed; $skippedCount phase(s) intentionally skipped. Evidence: $outputRoot" -ForegroundColor Yellow
}
else {
    Write-Host "`nAll requested gates passed. Evidence: $outputRoot" -ForegroundColor Green
}
exit 0
