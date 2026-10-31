# Window size and position

In normal mode, Yamantaka remembers the native window's normal position, size and maximized state. Entering fullscreen with F11, or minimizing the application, does not replace the saved normal layout. Leaving fullscreen returns control to the existing native window.

Window state lives in `window-state-v1.json` beside the independent Rust library. A background worker groups changes after 500 milliseconds without another placement change. The request queue holds one placement and shutdown flushes the most recent pending placement. This file has a single writer and does not compete with application-settings writes.

The current placement also enters the settings snapshot used for JSON backups. When restoring a backup into a store without a window-state file, Yamantaka can use its `nativeWindowPlacement` setting. Older imported Flutter settings retain their `windowWidth`, `windowHeight`, `windowX` and `windowY` values as the initial fallback. Rust does not change Flutter's preferences.

The native adapter pairs [GetWindowPlacement](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowplacement) with [SetWindowPlacement](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowplacement). Windows placement coordinates differ from screen coordinates; mixing them with screen-position APIs can move a window incorrectly. The restore API adjusts a fully off-screen placement when the screen configuration changes. Physical mixed-DPI and multi-monitor acceptance remains pending.

[Safe mode](safe-startup.md) skips restoring and recording window placement. Unreadable, oversized, unsupported or redirected window-state files are preserved and ignored. Yamantaka uses the initial window layout rather than replacing such a file. To reset that state manually, close the application and retain a renamed copy of `window-state-v1.json` before restarting.

The main window accepts `--software` or `--opengl` to request the existing Glow renderer. The default renderer remains wgpu with DX12. A successful software-renderer scenario does not establish compatibility with every GPU or device-loss recovery.
