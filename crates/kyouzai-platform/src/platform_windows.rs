use glam::Vec2;
use raw_window_handle::RawWindowHandle;

use crate::{PlatformError, PlatformWindowExt};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, ScreenToClient,
};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GWL_STYLE, GetForegroundWindow, GetPhysicalCursorPos, GetShellWindow,
    GetWindowLongPtrW, GetWindowRect, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_CAPTION, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};
use windows::core::{BOOL, PCWSTR};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const AUTOSTART_VALUE_NAME: &str = "KyouzaiCompanion";

pub struct WindowsPlatform {
    hwnd: HWND,
    click_through: bool,
}

impl WindowsPlatform {
    pub fn new(handle: RawWindowHandle) -> Result<Self, PlatformError> {
        let RawWindowHandle::Win32(win32) = handle else {
            return Err(PlatformError::Unsupported);
        };

        let hwnd = HWND(win32.hwnd.get() as *mut core::ffi::c_void);

        Ok(Self {
            hwnd,
            click_through: false,
        })
    }

    pub fn monitors(&self) -> Vec<crate::MonitorInfo> {
        let mut monitors = Vec::new();

        unsafe {
            let data = LPARAM(&mut monitors as *mut Vec<crate::MonitorInfo> as isize);

            let _ = EnumDisplayMonitors(None, None, Some(collect_monitor_info), data);
        }

        monitors
    }
}

unsafe extern "system" fn collect_monitor_info(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(data.0 as *mut Vec<crate::MonitorInfo>) };

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return BOOL(1);
    }

    let bounds = info.rcMonitor;
    let work = info.rcWork;

    monitors.push(crate::MonitorInfo {
        position: Vec2::new(bounds.left as f32, bounds.top as f32),
        size: Vec2::new(
            (bounds.right - bounds.left) as f32,
            (bounds.bottom - bounds.top) as f32,
        ),
        work_area_position: Vec2::new(work.left as f32, work.top as f32),
        work_area_size: Vec2::new(
            (work.right - work.left) as f32,
            (work.bottom - work.top) as f32,
        ),
    });

    BOOL(1)
}

unsafe extern "system" fn check_fullscreen_monitor(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let (window_rect, found) = unsafe { &mut *(data.0 as *mut (RECT, bool)) };

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool()
        && covers_monitor(window_rect, &info.rcMonitor)
    {
        *found = true;
        return BOOL(0);
    }

    BOOL(1)
}

fn covers_monitor(window: &RECT, monitor: &RECT) -> bool {
    window.left <= monitor.left
        && window.top <= monitor.top
        && window.right >= monitor.right
        && window.bottom >= monitor.bottom
}

impl PlatformWindowExt for WindowsPlatform {
    fn enable_layered(&mut self) -> Result<(), PlatformError> {
        unsafe {
            let current = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE);
            let updated = current | (WS_EX_LAYERED.0 as isize);

            SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, updated);

            let _ = SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
        Ok(())
    }

    fn monitors(&self) -> Vec<crate::MonitorInfo> {
        WindowsPlatform::monitors(self)
    }

    fn set_click_through(&mut self, enabled: bool) -> Result<(), PlatformError> {
        if self.click_through == enabled {
            return Ok(());
        }

        unsafe {
            let current = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE);
            let updated = if enabled {
                current | (WS_EX_TRANSPARENT.0 as isize)
            } else {
                current & !(WS_EX_TRANSPARENT.0 as isize)
            };
            SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, updated);
        }

        self.click_through = enabled;
        Ok(())
    }

    fn cursor_position(&self) -> Option<(f64, f64)> {
        let (x, y) = self.global_cursor_position()?;
        let mut point = POINT {
            x: x as i32,
            y: y as i32,
        };
        unsafe {
            if ScreenToClient(self.hwnd, &mut point).as_bool() {
                Some((point.x as f64, point.y as f64))
            } else {
                None
            }
        }
    }

    fn global_cursor_position(&self) -> Option<(f64, f64)> {
        unsafe {
            let mut point = POINT::default();
            GetPhysicalCursorPos(&mut point).ok()?;
            Some((point.x as f64, point.y as f64))
        }
    }

    fn is_fullscreen_app_foreground(&self) -> bool {
        unsafe {
            let foreground = GetForegroundWindow();

            if foreground.is_invalid() || foreground == self.hwnd || foreground == GetShellWindow()
            {
                return false;
            }

            let mut window_rect = RECT::default();

            if GetWindowRect(foreground, &mut window_rect).is_err() {
                return false;
            }

            let style = GetWindowLongPtrW(foreground, GWL_STYLE) as u32;
            let has_caption = (style & WS_CAPTION.0) != 0;

            if has_caption {
                return false;
            }

            let mut context = (window_rect, false);
            let data = LPARAM(&mut context as *mut (RECT, bool) as isize);

            let _ = EnumDisplayMonitors(None, None, Some(check_fullscreen_monitor), data);

            context.1
        }
    }

    fn register_autostart(&self, enabled: bool) -> Result<(), PlatformError> {
        if !enabled {
            return Err(PlatformError::Unsupported);
        }

        let exe_path =
            std::env::current_exe().map_err(|error| PlatformError::OsError(error.to_string()))?;

        let quoted_path = format!("\"{}\"", exe_path.display());
        let value_data: Vec<u16> = quoted_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let subkey: Vec<u16> = RUN_KEY.encode_utf16().chain(std::iter::once(0)).collect();
        let value_name: Vec<u16> = AUTOSTART_VALUE_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let byte_len = (value_data.len() * std::mem::size_of::<u16>()) as u32;

        let result = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(subkey.as_ptr()),
                PCWSTR(value_name.as_ptr()),
                REG_SZ.0,
                Some(value_data.as_ptr() as *const core::ffi::c_void),
                byte_len,
            )
        };

        result
            .ok()
            .map_err(|error| PlatformError::OsError(error.to_string()))
    }
}
