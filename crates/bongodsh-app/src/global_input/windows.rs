//! Isolated Win32 boundary. Hook, window, and callback state share one thread.
//! A separate hook avoids replacing winit's process-wide Raw Input registration.
#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]

use std::{
    cell::RefCell,
    ptr::{null, null_mut},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use bongodsh_core::InputEvent;
use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::{
        LibraryLoader::GetModuleHandleW,
        RemoteDesktop::{
            NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
            WTSUnRegisterSessionNotification,
        },
    },
    UI::{Input::KeyboardAndMouse::GetAsyncKeyState, WindowsAndMessaging::*},
};

use super::{
    Sink, Status,
    keys::{KeyFilter, windows_typing_key},
};

const WAKE: u32 = WM_APP + 1;
const MODIFIERS: [usize; 6] = [0xa2, 0xa3, 0xa4, 0xa5, 0x5b, 0x5c];

thread_local! {
    static STATE: RefCell<Option<Keyboard>> = const { RefCell::new(None) };
}

pub(super) fn has_permission() -> bool {
    true
}

fn physical_down(key: usize) -> bool {
    // SAFETY: This query accepts a virtual-key number and writes no memory.
    unsafe { GetAsyncKeyState(key as i32) < 0 }
}

struct Keyboard {
    sink: Sink,
    window: HWND,
    keys: KeyFilter,
    pressed: [bool; 512],
    virtual_keys: [usize; 512],
    virtual_down: [bool; 256],
    preheld: [bool; 256],
    changed: bool,
    sleeping: bool,
    locked: bool,
    status: Option<Status>,
}

impl Keyboard {
    fn new(sink: &Sink, window: HWND) -> Self {
        let down = std::array::from_fn(physical_down);
        Self {
            sink: sink.clone(),
            window,
            keys: KeyFilter::default(),
            pressed: [false; 512],
            virtual_keys: [0; 512],
            virtual_down: down,
            preheld: down,
            changed: false,
            sleeping: false,
            locked: false,
            status: None,
        }
    }

    fn reset(&mut self) {
        self.keys.reset();
        self.sink.input(InputEvent::Reset);
        self.changed = true;
    }

    fn lifecycle(&mut self) {
        self.reset();
        self.preheld = std::array::from_fn(physical_down);
        self.virtual_down = self.preheld;
        self.status = Some(if self.sleeping || self.locked {
            Status::Suspended
        } else {
            Status::Listening
        });
    }

    fn key(&mut self, event: &KBDLLHOOKSTRUCT) {
        if self.sleeping
            || self.locked
            || self.sink.stopped.load(Ordering::Acquire)
            || event.flags & LLKHF_INJECTED != 0
            || event.scanCode == 0
            || event.scanCode > 0xff
            || event.vkCode > 0xff
        {
            return;
        }
        let focused = self.sink.refresh_focus();
        if !focused {
            self.keys.reset();
        }
        let extended = event.flags & LLKHF_EXTENDED != 0;
        let code = event.scanCode as u16 | if extended { 0x100 } else { 0 };
        let index = usize::from(code);
        let vk = match event.vkCode {
            0x11 => {
                if extended {
                    0xa3
                } else {
                    0xa2
                }
            }
            0x12 => {
                if extended {
                    0xa5
                } else {
                    0xa4
                }
            }
            other => other as usize,
        };
        let pressed = event.flags & LLKHF_UP == 0;
        let repeat = self.pressed[index] || self.preheld[vk];
        self.pressed[index] = pressed;
        self.virtual_keys[index] = vk;
        self.virtual_down[vk] = pressed;
        if !pressed {
            self.preheld[vk] = false;
        }
        let shortcut =
            MODIFIERS.iter().any(|vk| self.virtual_down[*vk]) || event.flags & LLKHF_ALTDOWN != 0;
        if let Some(input) = self.keys.event(
            code,
            pressed,
            repeat,
            windows_typing_key(code) && focused,
            shortcut,
        ) {
            self.sink.input(input);
            self.changed = true;
        }
    }

    fn reconcile(&mut self) {
        for vk in 0..256 {
            let down = physical_down(vk);
            self.virtual_down[vk] = down;
            self.preheld[vk] &= down;
        }
        let released: Vec<_> = self
            .keys
            .tracked()
            .filter(|code| !self.virtual_down[self.virtual_keys[usize::from(*code)]])
            .collect();
        for code in released {
            self.pressed[usize::from(code)] = false;
            if let Some(input) = self.keys.event(code, false, false, true, false) {
                self.sink.input(input);
                self.changed = true;
            }
        }
        for (index, pressed) in self.pressed.iter_mut().enumerate() {
            *pressed &= self.virtual_down[self.virtual_keys[index]];
        }
    }
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && lparam != 0 {
        // SAFETY: WH_KEYBOARD_LL supplies KBDLLHOOKSTRUCT for HC_ACTION. The
        // pointer is read only during this callback and is never retained.
        let event = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        STATE.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(state) = slot.as_mut()
            {
                state.key(event);
                if state.changed {
                    // SAFETY: The hidden window lives until after unhooking. A
                    // posted message wakes our loop; it carries no key or text.
                    unsafe {
                        PostMessageW(state.window, WAKE, 0, 0);
                    }
                }
            }
        });
    }
    // SAFETY: Forward the original arguments for every code, including negatives.
    // Never swallow input, even when BongoDSH filters shortcuts or injected keys.
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if matches!(
        message,
        WM_POWERBROADCAST | WM_WTSSESSION_CHANGE | WM_DEVICECHANGE
    ) {
        STATE.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(state) = slot.as_mut()
            {
                match message {
                    WM_POWERBROADCAST => match wparam as u32 {
                        PBT_APMSUSPEND => state.sleeping = true,
                        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND | PBT_APMRESUMECRITICAL => {
                            state.sleeping = false
                        }
                        _ => return,
                    },
                    WM_WTSSESSION_CHANGE => match wparam as u32 {
                        WTS_SESSION_LOCK
                        | WTS_CONSOLE_DISCONNECT
                        | WTS_REMOTE_DISCONNECT
                        | WTS_SESSION_LOGOFF => state.locked = true,
                        WTS_SESSION_UNLOCK | WTS_CONSOLE_CONNECT | WTS_REMOTE_CONNECT
                        | WTS_SESSION_LOGON => state.locked = false,
                        _ => return,
                    },
                    _ => {}
                }
                state.lifecycle();
            }
        });
    }
    // SAFETY: All unhandled window messages retain their original parameters.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

struct Listener {
    instance: HINSTANCE,
    class: Vec<u16>,
    window: HWND,
    hook: HHOOK,
    session_notifications: bool,
}

impl Listener {
    fn install_hook(&mut self) -> Result<(), Status> {
        // SAFETY: The callback is a process-lifetime function. This owning thread
        // pumps messages until Drop unhooks it. No foreign pointers escape.
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), self.instance, 0) };
        if hook.is_null() {
            return Err(last_error("SetWindowsHookExW"));
        }
        if !self.hook.is_null() {
            // SAFETY: This handle belongs to this listener; replace only after
            // successful installation so a failed reconnect does not leak it.
            unsafe {
                UnhookWindowsHookEx(self.hook);
            }
        }
        self.hook = hook;
        Ok(())
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // SAFETY: All handles and the class belong to this thread. Unhook before
        // destroying the window or dropping TLS state used by callbacks.
        unsafe {
            if !self.hook.is_null() {
                UnhookWindowsHookEx(self.hook);
            }
            if !self.window.is_null() {
                KillTimer(self.window, 1);
                if self.session_notifications {
                    WTSUnRegisterSessionNotification(self.window);
                }
                DestroyWindow(self.window);
            }
            UnregisterClassW(self.class.as_ptr(), self.instance);
        }
        STATE.with(|slot| *slot.borrow_mut() = None);
    }
}

fn last_error(operation: &str) -> Status {
    Status::Failed(format!("{operation}: {}", std::io::Error::last_os_error()))
}

pub(super) fn listen(sink: &Sink) -> Result<(), Status> {
    let class: Vec<u16> = "BongoDSHKeyboardListener\0".encode_utf16().collect();
    // SAFETY: A null module name returns the executable's loaded module handle.
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err(last_error("GetModuleHandleW"));
    }
    let definition = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: class.as_ptr(),
        ..Default::default()
    };
    // SAFETY: Class name is NUL-terminated; the callback and module remain live.
    if unsafe { RegisterClassW(&definition) } == 0 {
        return Err(last_error("RegisterClassW"));
    }
    let mut listener = Listener {
        instance,
        class,
        window: null_mut(),
        hook: null_mut(),
        session_notifications: false,
    };
    // SAFETY: This is an invisible top-level window, not HWND_MESSAGE, so it
    // receives power and device broadcasts. All optional pointers are null.
    listener.window = unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            listener.class.as_ptr(),
            listener.class.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        )
    };
    if listener.window.is_null() {
        return Err(last_error("CreateWindowExW"));
    }
    // SAFETY: The listener owns a live window and unregisters it in Drop.
    if unsafe { WTSRegisterSessionNotification(listener.window, NOTIFY_FOR_THIS_SESSION) } == 0 {
        return Err(last_error("WTSRegisterSessionNotification"));
    }
    listener.session_notifications = true;
    // SAFETY: WM_TIMER is delivered to our window; no callback pointer is used.
    if unsafe { SetTimer(listener.window, 1, 250, None) } == 0 {
        return Err(last_error("SetTimer"));
    }
    STATE.with(|slot| *slot.borrow_mut() = Some(Keyboard::new(sink, listener.window)));
    listener.install_hook()?;
    sink.status(Status::Listening);
    let mut heartbeat = Instant::now();
    let mut next_check = heartbeat;
    while !sink.stopped.load(Ordering::Acquire) {
        let mut message = MSG::default();
        // SAFETY: MSG is writable and lives through dispatch; null HWND reads
        // this thread's queue, including the periodic shutdown/health timer.
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            return Err(last_error("GetMessageW"));
        }
        if result == 0 {
            break;
        }
        let now = Instant::now();
        if now.duration_since(heartbeat) > Duration::from_secs(1) {
            STATE.with(|slot| {
                if let Some(state) = slot.borrow_mut().as_mut() {
                    state.lifecycle();
                }
            });
            // Windows silently removes hooks that time out. Reinstall after a
            // stalled message loop or resume instead of retaining a stale hook.
            listener.install_hook()?;
        }
        heartbeat = now;
        // SAFETY: The message came from this thread's GetMessageW call. We never
        // TranslateMessage, so this listener never creates character messages.
        unsafe {
            DispatchMessageW(&message);
        }
        STATE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                if !sink.refresh_focus() {
                    state.keys.reset();
                }
                if now >= next_check {
                    next_check = now + Duration::from_secs(1);
                    state.reconcile();
                }
                if let Some(status) = state.status.take() {
                    sink.status(status);
                }
                if state.changed {
                    state.changed = false;
                    sink.wake();
                }
            }
        });
    }
    Ok(())
}
