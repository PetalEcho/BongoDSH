use std::time::{Duration, Instant};

use eframe::egui::{Pos2, Rect, pos2};
use raw_window_handle::RawWindowHandle;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        Graphics::{
            Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
            Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
        },
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
        UI::{
            Accessibility::{
                CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement,
                IUIAutomationValuePattern, UIA_ComboBoxControlTypeId, UIA_EditControlTypeId,
                UIA_ValuePatternId,
            },
            HiDpi::GetDpiForWindow,
            Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
            WindowsAndMessaging::{
                EnumWindows, GWL_EXSTYLE, GetWindowLongPtrW, GetWindowRect,
                GetWindowThreadProcessId, IsIconic, IsWindowVisible, SWP_NOACTIVATE, SWP_NOSIZE,
                SWP_NOZORDER, SetWindowPos, WS_EX_TOOLWINDOW,
            },
        },
    },
    core::{BOOL, Interface},
};

use super::{Request, Snapshot};
use crate::positioning::{
    AttachmentKind, Monitor, TargetWindow, Widget, WindowId, covers_screen, input_distance,
    pick_window,
};

pub(crate) struct NativeWindow(HWND);

impl NativeWindow {
    pub fn new(handle: RawWindowHandle) -> Option<Self> {
        let RawWindowHandle::Win32(handle) = handle else {
            return None;
        };
        Some(Self(HWND(handle.hwnd.get() as *mut _)))
    }

    pub fn geometry(&self) -> Option<(Widget, Vec<Monitor>)> {
        let mut bounds = RECT::default();
        let mut monitors = Vec::<Monitor>::new();
        // SAFETY: eframe's window is alive on this UI thread, and callbacks finish before the local vector is dropped.
        unsafe {
            GetWindowRect(self.0, &mut bounds).ok()?;
            EnumDisplayMonitors(
                None,
                None,
                Some(monitor_callback),
                LPARAM((&mut monitors as *mut Vec<Monitor>) as isize),
            )
            .ok()
            .ok()?;
            let scale = (GetDpiForWindow(self.0) as f32 / 96.0).max(1.0);
            Some((
                Widget {
                    outer: rect(bounds),
                    scale,
                },
                monitors,
            ))
        }
    }

    pub fn primary_down(&self) -> bool {
        // SAFETY: this reads only the left mouse button state; no pointers or keyboard text are involved.
        unsafe { GetAsyncKeyState(i32::from(VK_LBUTTON.0)) < 0 }
    }

    pub fn move_to(&self, position: Pos2) {
        // SAFETY: the live window belongs to eframe; no size, z-order or focus changes are requested.
        let _ = unsafe {
            SetWindowPos(
                self.0,
                None,
                position.x.round() as i32,
                position.y.round() as i32,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
            )
        };
    }
}

unsafe extern "system" fn monitor_callback(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: EnumDisplayMonitors supplies a valid monitor; data is our exclusive vector for this synchronous enumeration.
    unsafe {
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            let monitors = &mut *(data.0 as *mut Vec<Monitor>);
            monitors.push(Monitor {
                bounds: rect(info.rcMonitor),
                work_area: rect(info.rcWork),
                fullscreen_area: rect(info.rcMonitor),
            });
        }
    }
    BOOL(1)
}

fn rect(value: RECT) -> Rect {
    Rect::from_min_max(
        pos2(value.left as f32, value.top as f32),
        pos2(value.right as f32, value.bottom as f32),
    )
}

pub(crate) struct Probe {
    automation: Option<IUIAutomation>,
    input: Option<(WindowId, IUIAutomationElement)>,
    com_initialized: bool,
}

impl Probe {
    pub fn new() -> Self {
        // SAFETY: this new worker owns its MTA apartment and all UI Automation interfaces until Drop.
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).is_ok() };
        let automation: Option<IUIAutomation> = if initialized {
            // SAFETY: COM has been initialized on this thread; the CLSID and interface are system UI Automation types.
            unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER).ok() }
        } else {
            None
        };
        if let Some(api) = &automation
            && let Ok(settings) = api.cast::<IUIAutomation2>()
        {
            // SAFETY: these bounded timeouts affect only this client's UI Automation calls.
            unsafe {
                let _ = settings.SetConnectionTimeout(200);
                let _ = settings.SetTransactionTimeout(200);
            }
        }
        Self {
            automation,
            input: None,
            com_initialized: initialized,
        }
    }

    pub fn sample(&mut self, request: &Request) -> Snapshot {
        let mut windows = Vec::<TargetWindow>::new();
        // SAFETY: EnumWindows invokes callbacks synchronously with our local vector pointer.
        let _ = unsafe {
            EnumWindows(
                Some(window_callback),
                LPARAM((&mut windows as *mut Vec<TargetWindow>) as isize),
            )
        };
        for window in &mut windows {
            window.fullscreen = covers_screen(window.bounds, &request.monitors);
        }
        if !request.inputs {
            self.input = None;
        } else if request.snap {
            self.input = pick_window(request.widget, &windows).and_then(|host| {
                find_input(self.automation.as_ref()?, *host, request.widget)
                    .map(|element| (host.id, element))
            });
        } else if self
            .input
            .as_ref()
            .map(|(id, _)| (*id, AttachmentKind::Input))
            != request.follow
        {
            self.input = None;
        }
        let input = self.input.as_ref().and_then(|(id, element)| {
            let host = windows.iter().find(|window| window.id == *id)?;
            let bounds = input_bounds(element)?;
            host.bounds.intersects(bounds).then_some((*id, bounds))
        });
        Snapshot {
            generation: request.generation,
            snap: request.snap,
            windows,
            input,
            input_access: self.automation.is_some(),
        }
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        self.input = None;
        self.automation = None;
        if self.com_initialized {
            // SAFETY: balances this thread's successful CoInitializeEx after releasing every COM object.
            unsafe { CoUninitialize() };
        }
    }
}

unsafe extern "system" fn window_callback(handle: HWND, data: LPARAM) -> BOOL {
    // SAFETY: EnumWindows supplies the handle. The output pointer belongs to this synchronous enumeration.
    if let Some(window) = unsafe { read_window(handle) } {
        // SAFETY: data is the exclusive local Vec<TargetWindow> passed by sample.
        unsafe { (&mut *(data.0 as *mut Vec<TargetWindow>)).push(window) };
    }
    BOOL(1)
}

unsafe fn read_window(handle: HWND) -> Option<TargetWindow> {
    let mut process = 0;
    let mut cloaked = 0u32;
    let mut bounds = RECT::default();
    // SAFETY: Windows validates the enumerated handle, and every output points to correctly sized local storage.
    unsafe {
        GetWindowThreadProcessId(handle, Some(&mut process));
        if process == 0
            || process == std::process::id()
            || !IsWindowVisible(handle).as_bool()
            || IsIconic(handle).as_bool()
            || GetWindowLongPtrW(handle, GWL_EXSTYLE) & WS_EX_TOOLWINDOW.0 as isize != 0
        {
            return None;
        }
        let _ = DwmGetWindowAttribute(
            handle,
            DWMWA_CLOAKED,
            (&mut cloaked as *mut u32).cast(),
            size_of::<u32>() as u32,
        );
        if cloaked != 0 {
            return None;
        }
        if DwmGetWindowAttribute(
            handle,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&mut bounds as *mut RECT).cast(),
            size_of::<RECT>() as u32,
        )
        .is_err()
        {
            GetWindowRect(handle, &mut bounds).ok()?;
        }
    }
    let bounds = rect(bounds);
    (bounds.width() >= 80.0 && bounds.height() >= 50.0).then_some(TargetWindow {
        id: WindowId {
            handle: handle.0 as u64,
            process,
        },
        bounds,
        fullscreen: false,
    })
}

fn input_bounds(element: &IUIAutomationElement) -> Option<Rect> {
    // SAFETY: the retained interface is used only on the worker apartment that created it; stale elements return errors.
    unsafe {
        if element.CurrentIsOffscreen().ok()?.as_bool()
            || !element.CurrentIsEnabled().ok()?.as_bool()
        {
            return None;
        }
        let bounds = rect(element.CurrentBoundingRectangle().ok()?);
        (bounds.is_finite() && bounds.is_positive()).then_some(bounds)
    }
}

fn find_input(
    api: &IUIAutomation,
    host: TargetWindow,
    widget: Widget,
) -> Option<IUIAutomationElement> {
    // SAFETY: all interfaces remain in this worker's MTA. We query geometry and control metadata only.
    unsafe {
        let root = api.ElementFromHandle(HWND(host.id.handle as *mut _)).ok()?;
        let walker = api.ControlViewWalker().ok()?;
        let mut stack = vec![walker.GetFirstChildElement(&root).ok()?];
        let mut best: Option<(f32, IUIAutomationElement)> = None;
        let deadline = Instant::now() + Duration::from_millis(180);
        for _ in 0..256 {
            if Instant::now() >= deadline {
                break;
            }
            let Some(element) = stack.pop() else { break };
            if let Ok(sibling) = walker.GetNextSiblingElement(&element) {
                stack.push(sibling);
            }
            let bounds = input_bounds(&element);
            if bounds.is_some_and(|bounds| {
                !bounds
                    .expand(48.0 * widget.scale)
                    .intersects(widget.content())
            }) {
                continue;
            }
            let kind = element.CurrentControlType().ok();
            let writable = element
                .GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
                .ok()
                .and_then(|pattern| pattern.CurrentIsReadOnly().ok())
                .is_none_or(|read_only| !read_only.as_bool());
            if (kind == Some(UIA_EditControlTypeId) || kind == Some(UIA_ComboBoxControlTypeId))
                && writable
                && let Some(distance) = bounds.and_then(|bounds| input_distance(widget, bounds))
                && best
                    .as_ref()
                    .is_none_or(|(previous, _)| distance < *previous)
            {
                best = Some((distance, element));
            } else if let Ok(child) = walker.GetFirstChildElement(&element) {
                stack.push(child);
            }
        }
        best.map(|(_, element)| element)
    }
}
