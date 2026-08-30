[CmdletBinding()]
param(
    [string]$BaselineDirectory = 'build/performance/0.10.0-baseline',
    [string]$CurrentDirectory = 'build/performance/0.11.0-current',
    [string]$OutputPath = 'build/performance/0.11.0-current/comparison-0.10-vs-0.11.json',
    [ValidateRange(0, 1)][double]$Tolerance = 0.10
)

$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

function Resolve-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    if ([System.IO.Path]::IsPathRooted($Path)) { return [System.IO.Path]::GetFullPath($Path) }
    return [System.IO.Path]::GetFullPath((Join-Path $root $Path))
}

function Read-Passes {
    param(
        [Parameter(Mandatory = $true)][string]$Directory,
        [Parameter(Mandatory = $true)][string]$Pattern
    )
    $files = @(Get-ChildItem -LiteralPath $Directory -Filter $Pattern -File | Sort-Object Name)
    if ($files.Count -ne 5) { throw "Expected five files matching $Pattern under $Directory; found $($files.Count)." }
    return @($files | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json })
}

function Get-Median {
    param([Parameter(Mandatory = $true)][double[]]$Values)
    $sorted = @($Values | Sort-Object)
    return [double]$sorted[[Math]::Floor($sorted.Count / 2)]
}

$baselineRoot = Resolve-RepositoryPath $BaselineDirectory
$currentRoot = Resolve-RepositoryPath $CurrentDirectory
$output = Resolve-RepositoryPath $OutputPath
$baselineAdvanced = Read-Passes -Directory $baselineRoot -Pattern 'advanced-pass-*.json'
$currentAdvanced = Read-Passes -Directory $currentRoot -Pattern 'advanced-pass-*.json'
$baselineLibrary = Read-Passes -Directory $baselineRoot -Pattern 'library-pass-*.json'
$currentLibrary = Read-Passes -Directory $currentRoot -Pattern 'library-pass-*.json'
$baselineNative = Read-Passes -Directory $baselineRoot -Pattern 'native-profile-pass-*.json'
$currentNative = Read-Passes -Directory $currentRoot -Pattern 'native-profile-pass-*.json'
$comparisons = [System.Collections.Generic.List[object]]::new()

function Add-LowerIsBetterComparison {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][double[]]$Baseline,
        [Parameter(Mandatory = $true)][double[]]$Current,
        [Parameter(Mandatory = $true)][string]$Unit
    )
    $baselineMedian = Get-Median $Baseline
    $currentMedian = Get-Median $Current
    $delta = if ($baselineMedian -eq 0) { $null } else { (($currentMedian - $baselineMedian) / $baselineMedian) * 100 }
    $limit = $baselineMedian * (1 + $Tolerance)
    $regressedSamples = @($Current | Where-Object { $_ -gt $limit }).Count
    $medianRegression = $currentMedian -gt $limit
    $reliableRegression = $medianRegression -and $regressedSamples -ge 4
    $comparisons.Add([ordered]@{
            metric = $Name
            unit = $Unit
            baselineSamples = $Baseline
            currentSamples = $Current
            baselineMedian = $baselineMedian
            currentMedian = $currentMedian
            baselineMinimum = ($Baseline | Measure-Object -Minimum).Minimum
            baselineMaximum = ($Baseline | Measure-Object -Maximum).Maximum
            currentMinimum = ($Current | Measure-Object -Minimum).Minimum
            currentMaximum = ($Current | Measure-Object -Maximum).Maximum
            deltaPercent = if ($null -eq $delta) { $null } else { [Math]::Round($delta, 3) }
            limit = $limit
            currentSamplesAboveLimit = $regressedSamples
            medianRegression = $medianRegression
            reliableRegression = $reliableRegression
            status = if ($reliableRegression) { 'fail' } elseif ($medianRegression) { 'watch' } else { 'pass' }
        })
}

Add-LowerIsBetterComparison 'advanced.bulk_insert' @($baselineAdvanced | ForEach-Object { $_.bulk_insert_ms }) @($currentAdvanced | ForEach-Object { $_.bulk_insert_ms }) 'ms'
foreach ($metric in @('global_search_us', 'collection_query_us', 'workspace_restore_us')) {
    foreach ($stat in @('median', 'p95')) {
        Add-LowerIsBetterComparison "advanced.$metric.$stat" @($baselineAdvanced | ForEach-Object { $_.$metric.$stat }) @($currentAdvanced | ForEach-Object { $_.$metric.$stat }) 'us'
    }
}

foreach ($count in @(5000, 25000, 100000)) {
    foreach ($metric in @('firstLoadUs', 'queryMedianUs', 'queryP95Us')) {
        Add-LowerIsBetterComparison "library.$count.$metric" @($baselineLibrary | ForEach-Object { $pass = $_; ($pass.results | Where-Object { $_.documents -eq $count }).$metric }) @($currentLibrary | ForEach-Object { $pass = $_; ($pass.results | Where-Object { $_.documents -eq $count }).$metric }) 'us'
    }
}

foreach ($metric in @(
        'library_median', 'library_p95', 'reader_median', 'reader_p95',
        'first_page_visible_100_pages', 'first_page_visible_5000_pages',
        'layout_change_p95', 'continuous_navigation_p95', 'continuous_scroll', 'resize_p95'
    )) {
    Add-LowerIsBetterComparison "native.interaction.$metric" @($baselineNative | ForEach-Object { $_.interaction_microseconds.$metric }) @($currentNative | ForEach-Object { $_.interaction_microseconds.$metric }) 'us'
}
foreach ($metric in @('heavy_open_median', 'heavy_open_p95', 'heavy_render_median', 'heavy_render_p95')) {
    Add-LowerIsBetterComparison "native.pdf_engine.$metric" @($baselineNative | ForEach-Object { $_.pdf_engine_microseconds.$metric }) @($currentNative | ForEach-Object { $_.pdf_engine_microseconds.$metric }) 'us'
}
foreach ($metric in @('library_p95', 'reader_p95')) {
    Add-LowerIsBetterComparison "native.frame_build.$metric" @($baselineNative | ForEach-Object { $_.frame_build_microseconds.$metric }) @($currentNative | ForEach-Object { $_.frame_build_microseconds.$metric }) 'us'
}
foreach ($metric in @('peak', 'after_return_minimum', 'heavy_close_growth')) {
    Add-LowerIsBetterComparison "native.memory.$metric" @($baselineNative | ForEach-Object { $_.memory_bytes.$metric }) @($currentNative | ForEach-Object { $_.memory_bytes.$metric }) 'bytes'
}

$invariants = [System.Collections.Generic.List[object]]::new()
function Add-Invariant {
    param([string]$Name, [bool]$Passed, [object]$Actual)
    $invariants.Add([ordered]@{ name = $Name; actual = $Actual; status = if ($Passed) { 'pass' } else { 'fail' } })
}
$allLibraryRows = @($currentLibrary | ForEach-Object { $_.results })
Add-Invariant 'library.materialized_at_most_80' (@($allLibraryRows | Where-Object { $_.materializedDocuments -gt 80 }).Count -eq 0) (@($allLibraryRows | Measure-Object materializedDocuments -Maximum).Maximum)
Add-Invariant 'library.zero_initial_file_system_walks' (@($allLibraryRows | Where-Object { $_.firstLoadFileSystemWalks -ne 0 }).Count -eq 0) (@($allLibraryRows | Measure-Object firstLoadFileSystemWalks -Maximum).Maximum)
Add-Invariant 'library.integrity_ok' (@($allLibraryRows | Where-Object { $_.integrity -ne 'ok' }).Count -eq 0) 'five passes x three corpora'
Add-Invariant 'advanced.no_private_paths_or_titles' (@($currentAdvanced | Where-Object { $_.contains_paths_or_titles }).Count -eq 0) 'five passes'
Add-Invariant 'advanced.annotations_100' (@($currentAdvanced | Where-Object { $_.fixtures.annotations -ne 100 }).Count -eq 0) 100
Add-Invariant 'advanced.workspace_tabs_5' (@($currentAdvanced | Where-Object { $_.fixtures.workspace_tabs -ne 5 }).Count -eq 0) 5
Add-Invariant 'native.zero_open_documents_after_close' (@($currentNative | Where-Object { $_.pdf_engine_diagnostics.open_documents -ne 0 }).Count -eq 0) (@($currentNative | ForEach-Object { $_.pdf_engine_diagnostics.open_documents }))
Add-Invariant 'native.zero_pending_renders_after_close' (@($currentNative | Where-Object { $_.pdf_engine_diagnostics.pending_renders -ne 0 }).Count -eq 0) (@($currentNative | ForEach-Object { $_.pdf_engine_diagnostics.pending_renders }))
Add-Invariant 'native.zero_cached_pages_after_close' (@($currentNative | Where-Object { $_.pdf_engine_diagnostics.cached_pages -ne 0 }).Count -eq 0) (@($currentNative | ForEach-Object { $_.pdf_engine_diagnostics.cached_pages }))

$failedMetrics = @($comparisons | Where-Object { $_.status -eq 'fail' })
$watchedMetrics = @($comparisons | Where-Object { $_.status -eq 'watch' })
$failedInvariants = @($invariants | Where-Object { $_.status -eq 'fail' })
$report = [ordered]@{
    format = 'yamantaka-iteration-performance-comparison'
    version = 1
    generatedAt = [DateTime]::UtcNow.ToString('o')
    host = [ordered]@{ operatingSystem = [System.Environment]::OSVersion.VersionString; processorCount = [Environment]::ProcessorCount }
    baselineVersion = '0.10.0'
    currentVersion = '0.11.0'
    passes = 5
    tolerance = $Tolerance
    reliabilityRule = 'A regression is reliable when the median exceeds the 0.10 median by more than the tolerance and at least four of five current samples exceed that limit. A median-only movement with three or fewer samples above the limit is retained as watch because the ranges overlap.'
    comparisons = @($comparisons)
    invariants = @($invariants)
    failedMetricCount = $failedMetrics.Count
    watchedMetricCount = $watchedMetrics.Count
    failedInvariantCount = $failedInvariants.Count
    status = if ($failedMetrics.Count -ne 0 -or $failedInvariants.Count -ne 0) { 'fail' } elseif ($watchedMetrics.Count -ne 0) { 'pass-with-watch' } else { 'pass' }
}
New-Item -ItemType Directory -Force -Path (Split-Path $output -Parent) | Out-Null
$report | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 -LiteralPath $output
Write-Host "Iteration performance comparison: $output ($($report.status))"
if ($report.status -eq 'fail') { exit 1 }
