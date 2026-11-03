#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]

use eframe::egui::{Pos2, Rect, pos2};
use raw_window_handle::RawWindowHandle;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::{
        Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
        Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    },
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON},
        WindowsAndMessaging::{
            EnumWindows, GWL_EXSTYLE, GetCursorPos, GetSystemMetrics, GetWindowLongPtrW,
            GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed,
            SM_SWAPBUTTON, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
            WS_EX_TOOLWINDOW,
        },
    },
};

use crate::positioning::{Monitor, TargetWindow, Widget, WindowId};

pub(crate) struct NativeWindow(HWND);

impl NativeWindow {
    pub fn new(handle: RawWindowHandle) -> Option<Self> {
        let RawWindowHandle::Win32(handle) = handle else {
            return None;
        };
        Some(Self(handle.hwnd.get() as HWND))
    }

    pub fn geometry(&self, content: Rect) -> Option<(Widget, Vec<Monitor>)> {
        let mut bounds = RECT::default();
        let mut monitors = Vec::<Monitor>::new();
        // SAFETY: eframe owns a live HWND; the synchronous callback borrows this vector only during enumeration.
        unsafe {
            if GetWindowRect(self.0, &mut bounds) == 0 {
                return None;
            }
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                Some(monitor_callback),
                (&mut monitors as *mut Vec<Monitor>) as LPARAM,
            );
            let scale = (GetDpiForWindow(self.0) as f32 / 96.0).max(1.0);
            let outer = rect(bounds);
            Some((
                Widget {
                    outer,
                    content: Rect::from_min_max(
                        outer.min + content.min.to_vec2() * scale,
                        outer.min + content.max.to_vec2() * scale,
                    ),
                    scale,
                },
                monitors,
            ))
        }
    }

    pub fn cursor(&self) -> Option<Pos2> {
        let mut point = POINT::default();
        // SAFETY: point is initialized writable storage. This reads only the mouse position.
        (unsafe { GetCursorPos(&mut point) } != 0).then(|| pos2(point.x as f32, point.y as f32))
    }

    pub fn primary_down(&self) -> bool {
        // SAFETY: neither call takes pointers. GetAsyncKeyState uses physical buttons,
        // so select the user's configured primary button before reading its state.
        unsafe {
            let button = if GetSystemMetrics(SM_SWAPBUTTON) != 0 {
                VK_RBUTTON
            } else {
                VK_LBUTTON
            };
            GetAsyncKeyState(i32::from(button)) < 0
        }
    }

    pub fn move_to(&self, position: Pos2) {
        // SAFETY: eframe owns this live window. Positioning preserves focus, dimensions and z-order.
        unsafe {
            SetWindowPos(
                self.0,
                std::ptr::null_mut(),
                position.x.round() as i32,
                position.y.round() as i32,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
            );
        }
    }
}

unsafe extern "system" fn monitor_callback(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> i32 {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: EnumDisplayMonitors provides the handle; data points to the caller's exclusive vector for this call.
    unsafe {
        if GetMonitorInfoW(monitor, &mut info) != 0 {
            (&mut *(data as *mut Vec<Monitor>)).push(Monitor {
                bounds: rect(info.rcMonitor),
                work_area: rect(info.rcWork),
            });
        }
    }
    1
}

fn rect(value: RECT) -> Rect {
    Rect::from_min_max(
        pos2(value.left as f32, value.top as f32),
        pos2(value.right as f32, value.bottom as f32),
    )
}

pub(crate) fn visible_windows() -> Vec<TargetWindow> {
    let mut windows = Vec::new();
    // SAFETY: enumeration is synchronous and data remains an exclusive, live vector until it returns.
    unsafe {
        EnumWindows(
            Some(window_callback),
            (&mut windows as *mut Vec<TargetWindow>) as LPARAM,
        );
    }
    windows
}

unsafe extern "system" fn window_callback(window: HWND, data: LPARAM) -> i32 {
    let mut process = 0;
    let mut cloaked = 0u32;
    let mut bounds = RECT::default();
    // SAFETY: EnumWindows supplies a candidate HWND. Every API validates stale handles, and output pointers refer to initialized local storage.
    unsafe {
        GetWindowThreadProcessId(window, &mut process);
        if process == std::process::id()
            || IsWindowVisible(window) == 0
            || IsIconic(window) != 0
            || GetWindowLongPtrW(window, GWL_EXSTYLE) & WS_EX_TOOLWINDOW as isize != 0
        {
            return 1;
        }
        let _ = DwmGetWindowAttribute(
            window,
            DWMWA_CLOAKED as u32,
            (&mut cloaked as *mut u32).cast(),
            size_of::<u32>() as u32,
        );
        if cloaked != 0 {
            return 1;
        }
        if DwmGetWindowAttribute(
            window,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            (&mut bounds as *mut RECT).cast(),
            size_of::<RECT>() as u32,
        ) < 0
            && GetWindowRect(window, &mut bounds) == 0
        {
            return 1;
        }
        let bounds = rect(bounds);
        if bounds.width() >= 80.0 && bounds.height() >= 50.0 {
            (&mut *(data as *mut Vec<TargetWindow>)).push(TargetWindow {
                id: WindowId {
                    handle: window as u64,
                    process,
                },
                bounds,
                maximized: IsZoomed(window) != 0,
            });
        }
    }
    1
}

pub(crate) fn has_focus_permission() -> bool {
    true
}

pub(crate) fn window_is_focused(target: WindowId) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };
    // SAFETY: Both queries use OS-owned handles; pid is a writable local. No window is mutated.
    unsafe {
        let foreground = GetForegroundWindow();
        let mut process = 0;
        !foreground.is_null()
            && foreground as usize as u64 == target.handle
            && GetWindowThreadProcessId(foreground, &mut process) != 0
            && process == target.process
    }
}
