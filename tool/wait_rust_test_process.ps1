function Wait-RustTestProcess {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][System.Diagnostics.Process]$Process,
        [ValidateRange(1, 300000)][int]$TimeoutMilliseconds = 45000
    )
    try {
        if (-not $Process.WaitForExit($TimeoutMilliseconds)) {
            throw 'The native test process timed out.'
        }
    } finally {
        # Retain the exact child handle; a reused PID must never select another app.
        if (-not $Process.HasExited) {
            $Process.Kill()
            if (-not $Process.WaitForExit(5000)) {
                throw 'The native test process has not exited after cleanup.'
            }
        }
    }
}
