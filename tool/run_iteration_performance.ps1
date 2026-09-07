[CmdletBinding()]
param([string]$OutputDirectory = 'build/performance/0.15.0-current')

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output = [IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
if (-not $output.StartsWith((Join-Path $root 'build') + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Performance output must remain below build.'
}
New-Item -ItemType Directory -Force -Path $output | Out-Null
$env:FLUTTER_SUPPRESS_ANALYTICS = 'true'
$env:DART_SUPPRESS_ANALYTICS = 'true'
$flutter = Join-Path $root '.flutter-sdk/bin/flutter.bat'
$dart = Join-Path $root '.flutter-sdk/bin/dart.bat'
Push-Location $root
try {
    for ($pass = 1; $pass -le 5; $pass++) {
        Write-Host "Pass $pass/5: advanced features"
        & $dart run tool/benchmark_advanced_features.dart (Join-Path $output "advanced-pass-$pass.json")
        if ($LASTEXITCODE -ne 0) { throw "Advanced benchmark failed on pass $pass." }
        Write-Host "Pass $pass/5: library"
        & $dart run tool/benchmark_library.dart (Join-Path $output "library-pass-$pass.json")
        if ($LASTEXITCODE -ne 0) { throw "Library benchmark failed on pass $pass." }
        Write-Host "Pass $pass/5: native PDF profile"
        $started = [DateTime]::UtcNow
        $ErrorActionPreference = 'Continue'
        & $flutter drive --driver=test_driver/performance_driver.dart --target=integration_test/performance_profile_test.dart -d windows --profile --no-pub *> (Join-Path $output "native-profile-pass-$pass.log")
        $ErrorActionPreference = 'Stop'
        if ($LASTEXITCODE -ne 0) { throw "Native profile failed on pass $pass. See its log." }
        $profile = Get-Item -LiteralPath (Join-Path $root 'build/performance/performance-profile.json')
        if ($profile.LastWriteTimeUtc -lt $started) { throw 'Native profile result is stale.' }
        Copy-Item -LiteralPath $profile.FullName -Destination (Join-Path $output "native-profile-pass-$pass.json")
    }
    & (Join-Path $PSScriptRoot 'compare_iteration_performance.ps1') -CurrentDirectory $OutputDirectory
    if ($LASTEXITCODE -ne 0) { throw 'Performance comparison failed; preserve and inspect the measured samples.' }
}
finally { Pop-Location }
