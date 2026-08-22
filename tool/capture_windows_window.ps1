param(
    [Parameter(Mandatory = $true)]
    [int]$TargetProcessId,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath,

    [int]$Width = 1200,
    [int]$Height = 800,
    [int]$X = 80,
    [int]$Y = 80,
    [int]$ClickX = -1,
    [int]$ClickY = -1,
    [switch]$PreserveWindowBounds,
    [string]$SendKeys = ''
)

Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class YamantakaWindowCapture {
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")]
    public static extern bool SetProcessDPIAware();

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr handle, out Rect rect);

    [DllImport("user32.dll")]
    public static extern bool MoveWindow(
        IntPtr handle,
        int x,
        int y,
        int width,
        int height,
        bool repaint
    );

    [DllImport("user32.dll")]
    public static extern bool PrintWindow(
        IntPtr handle,
        IntPtr deviceContext,
        uint flags
    );

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr handle);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr handle, int command);

    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);

    [DllImport("user32.dll")]
    public static extern void mouse_event(
        uint flags,
        uint dx,
        uint dy,
        uint data,
        UIntPtr extraInfo
    );
}
'@

[void][YamantakaWindowCapture]::SetProcessDPIAware()

$process = [System.Diagnostics.Process]::GetProcessById($TargetProcessId)
$deadline = [DateTime]::UtcNow.AddSeconds(15)
do {
    $process.Refresh()
    $handle = $process.MainWindowHandle
    if ($handle -ne [IntPtr]::Zero) {
        break
    }
    Start-Sleep -Milliseconds 100
} while ([DateTime]::UtcNow -lt $deadline)

if ($handle -eq [IntPtr]::Zero) {
    throw "The process did not expose a main window."
}

if (-not $PreserveWindowBounds) {
    [void][YamantakaWindowCapture]::ShowWindow($handle, 9)
    [void][YamantakaWindowCapture]::MoveWindow(
        $handle,
        $X,
        $Y,
        $Width,
        $Height,
        $true
    )
}
[void][YamantakaWindowCapture]::SetForegroundWindow($handle)
# Native startup includes database hydration and optional command-line opening.
# Wait for the routed screen, not merely for the Win32 handle to exist.
Start-Sleep -Milliseconds 2500

if ($ClickX -ge 0 -and $ClickY -ge 0) {
    $rect = New-Object YamantakaWindowCapture+Rect
    if (-not [YamantakaWindowCapture]::GetWindowRect($handle, [ref]$rect)) {
        throw "Could not read the window bounds before clicking."
    }
    [void][YamantakaWindowCapture]::SetCursorPos(
        $rect.Left + $ClickX,
        $rect.Top + $ClickY
    )
    # Reader chrome may be hidden until pointer movement. Give Flutter one
    # frame window to reveal and lay it out before pressing the control.
    Start-Sleep -Milliseconds 1200
    [YamantakaWindowCapture]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
    [YamantakaWindowCapture]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 1200
}

if ($SendKeys) {
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.SendKeys]::SendWait($SendKeys)
    Start-Sleep -Milliseconds 1200
}

$process.Refresh()
if ($process.HasExited) {
    throw "The application exited before its window could be captured (exit code $($process.ExitCode))."
}

$windowRect = New-Object YamantakaWindowCapture+Rect
if (-not [YamantakaWindowCapture]::GetWindowRect($handle, [ref]$windowRect)) {
    throw "Could not read the window bounds."
}

$captureWidth = $windowRect.Right - $windowRect.Left
$captureHeight = $windowRect.Bottom - $windowRect.Top
$bitmap = New-Object System.Drawing.Bitmap($captureWidth, $captureHeight)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$deviceContext = $graphics.GetHdc()
try {
    if (-not [YamantakaWindowCapture]::PrintWindow(
        $handle,
        $deviceContext,
        2
    )) {
        throw "PrintWindow failed."
    }
}
finally {
    $graphics.ReleaseHdc($deviceContext)
    $graphics.Dispose()
}

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$outputDirectory = [System.IO.Path]::GetDirectoryName($resolvedOutput)
[System.IO.Directory]::CreateDirectory($outputDirectory) | Out-Null
try {
    $bitmap.Save(
        $resolvedOutput,
        [System.Drawing.Imaging.ImageFormat]::Png
    )
}
finally {
    $bitmap.Dispose()
}

Write-Output $resolvedOutput
