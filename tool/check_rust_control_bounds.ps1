param(
    [Parameter(Mandatory=$true)][string[]]$ControlFiles,
    [string[]]$VisibleLabels = @()
)
$ErrorActionPreference = 'Stop'
$tolerance = 0.5
foreach ($controlFile in $ControlFiles) {
    $capture = Get-Content -LiteralPath $controlFile -Raw -Encoding UTF8 | ConvertFrom-Json
    if (-not $capture.synthetic -or @($capture.controls).Count -eq 0) {
        throw 'Expected a synthetic native capture with recorded controls.'
    }
    foreach ($control in $capture.controls) {
        if (@($control.rect).Count -ne 4 -or @($control.clip).Count -ne 4) { throw 'Invalid control geometry.' }
        foreach ($coordinate in (@($control.rect) + @($control.clip))) {
            if ([double]::IsNaN([double]$coordinate) -or [double]::IsInfinity([double]$coordinate)) { throw 'Non-finite control geometry.' }
        }
        if ($control.rect[0] -lt $control.clip[0] - $tolerance -or $control.rect[2] -gt $control.clip[2] + $tolerance) {
            throw "Control exceeds horizontal clip: $($control.label)"
        }
    }
    foreach ($label in $VisibleLabels) {
        $visible = @($capture.controls | Where-Object {
            $_.label -ceq $label -and $_.enabled -and
            $_.rect[0] -ge $_.clip[0] - $tolerance -and $_.rect[2] -le $_.clip[2] + $tolerance -and
            $_.rect[1] -ge $_.clip[1] - $tolerance -and $_.rect[3] -le $_.clip[3] + $tolerance
        })
        if ($visible.Count -eq 0) { throw "Required enabled control is not fully visible: $label" }
    }
    Write-Output "Control bounds verified: $([IO.Path]::GetFileName($controlFile))"
}
