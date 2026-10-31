//! Native workspace coordinates must round-trip through the same Windows API.
use raw_window_handle::{RawWindowHandle, WindowHandle};
use serde_json::{Value, json};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        GetWindowPlacement, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, SW_SHOWNORMAL, SetWindowPlacement,
        WINDOWPLACEMENT, WPF_RESTORETOMAXIMIZED,
    },
};
use yamantaka_domain::{Failure, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    normal: [i32; 4],
    maximized: bool,
}
impl Placement {
    pub fn normal(self) -> [i32; 4] {
        self.normal
    }
    pub fn maximized(self) -> bool {
        self.maximized
    }
    pub fn decode(value: &Value) -> Option<Self> {
        if value.as_object()?.len() != 3 {
            return None;
        }
        if value.get("version")?.as_u64()? != 1 {
            return None;
        }
        let rect = value.get("normal")?.as_array()?;
        if rect.len() != 4 {
            return None;
        }
        let mut normal = [0; 4];
        for (target, value) in normal.iter_mut().zip(rect) {
            *target = i32::try_from(value.as_i64()?).ok()?;
            if target.unsigned_abs() > 1_000_000 {
                return None;
            }
        }
        let width = i64::from(normal[2]) - i64::from(normal[0]);
        let height = i64::from(normal[3]) - i64::from(normal[1]);
        if !(64..=65_536).contains(&width) || !(64..=65_536).contains(&height) {
            return None;
        }
        Some(Self {
            normal,
            maximized: value.get("maximized")?.as_bool()?,
        })
    }
    pub fn encode(self) -> Value {
        json!({"version":1,"normal":self.normal,"maximized":self.maximized})
    }
    pub fn capture(handle: WindowHandle<'_>) -> Result<Self> {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        // The borrowed handle keeps the caller's native window alive for this call.
        unsafe { GetWindowPlacement(hwnd(handle)?, &mut placement) }
            .map_err(super::platform_error)?;
        let rect = placement.rcNormalPosition;
        let maximized = placement.showCmd == SW_SHOWMAXIMIZED.0 as u32
            || (placement.showCmd == SW_SHOWMINIMIZED.0 as u32
                && placement.flags.0 & WPF_RESTORETOMAXIMIZED.0 != 0);
        Self::decode(&json!({"version":1,"normal":[rect.left,rect.top,rect.right,rect.bottom],"maximized":maximized}))
            .ok_or_else(|| Failure::Invalid("Invalid native window placement".into()))
    }
    pub fn restore(self, handle: WindowHandle<'_>) -> Result<()> {
        let [left, top, right, bottom] = self.normal;
        let placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            rcNormalPosition: RECT {
                left,
                top,
                right,
                bottom,
            },
            showCmd: if self.maximized {
                SW_SHOWMAXIMIZED.0
            } else {
                SW_SHOWNORMAL.0
            } as u32,
            ..Default::default()
        };
        // Windows adjusts a completely off-screen placement to the current screens.
        unsafe { SetWindowPlacement(hwnd(handle)?, &placement) }.map_err(super::platform_error)
    }
}
fn hwnd(handle: WindowHandle<'_>) -> Result<HWND> {
    match handle.as_raw() {
        RawWindowHandle::Win32(window) => Ok(HWND(window.hwnd.get() as *mut std::ffi::c_void)),
        _ => Err(Failure::Invalid(
            "A native Windows handle is required".into(),
        )),
    }
}
