//! The only macOS FFI boundary. Callbacks and native objects stay on one thread.
#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]

use std::{
    ffi::c_void,
    ptr::{NonNull, null_mut},
    sync::atomic::Ordering,
    time::{Duration, Instant, SystemTime},
};

use bongodsh_core::InputEvent;
use objc2_core_foundation::{
    CFMachPort, CFRetained, CFRunLoop, CFRunLoopSource, kCFRunLoopDefaultMode,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
    CGPreflightListenEventAccess, CGRequestListenEventAccess,
};

use super::{
    Sink, Status,
    keys::{KeyFilter, mac_typing_key},
};

// SAFETY: Apple's CarbonEventsCore.h declares this no-argument Boolean query.
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn IsSecureEventInputEnabled() -> u8;
}

const CAN_SLEEP: u32 = 0xe000_0270;
const WILL_SLEEP: u32 = 0xe000_0280;
const DID_WAKE: u32 = 0xe000_0300;

// SAFETY: Signatures match IOKitLib.h and IOPMLib.h. IOKit handles are u32,
// notification ports are opaque pointers, and power-change tokens are intptr_t.
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IORegisterForSystemPower(
        context: *mut c_void,
        port: *mut *mut c_void,
        callback: unsafe extern "C" fn(*mut c_void, u32, u32, *mut c_void),
        notifier: *mut u32,
    ) -> u32;
    fn IODeregisterForSystemPower(notifier: *mut u32) -> i32;
    fn IOAllowPowerChange(connection: u32, token: isize) -> i32;
    fn IOServiceClose(connection: u32) -> i32;
    fn IONotificationPortGetRunLoopSource(port: *mut c_void) -> *const CFRunLoopSource;
    fn IONotificationPortDestroy(port: *mut c_void);
}

pub(super) fn has_permission() -> bool {
    CGPreflightListenEventAccess()
}

pub(super) fn request_permission() -> bool {
    CGRequestListenEventAccess()
}

pub(super) fn open_permission_settings() -> Result<(), String> {
    std::process::Command::new("/usr/bin/open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
        .status()
        .map_err(|error| error.to_string())?
        .success()
        .then_some(())
        .ok_or_else(|| "Cannot open System Settings".to_owned())
}

fn secure_input() -> bool {
    // SAFETY: This is a process-independent OS status query with no pointers.
    unsafe { IsSecureEventInputEnabled() != 0 }
}

struct Callback<'a> {
    sink: &'a Sink,
    keys: KeyFilter,
    heartbeat: SystemTime,
    changed: bool,
    suspended: bool,
    sleeping: bool,
    power_connection: u32,
}

impl Callback<'_> {
    fn reset(&mut self) {
        self.keys.reset();
        self.sink.input(InputEvent::Reset);
        self.changed = true;
    }

    fn check_sleep_gap(&mut self) {
        let now = SystemTime::now();
        // SystemTime includes sleep on macOS; Instant may stop while asleep.
        if now
            .duration_since(self.heartbeat)
            .map_or(true, |gap| gap > Duration::from_secs(2))
        {
            self.reset();
        }
        self.heartbeat = now;
    }
}

unsafe extern "C-unwind" fn on_event(
    _proxy: CGEventTapProxy,
    kind: CGEventType,
    event: NonNull<CGEvent>,
    user: *mut c_void,
) -> *mut CGEvent {
    // SAFETY: listen keeps this Box alive until the tap and source are invalidated.
    // The private run loop invokes this callback serially on its owning thread.
    let state = unsafe { &mut *user.cast::<Callback<'_>>() };
    if state.sink.stopped.load(Ordering::Acquire) {
        return event.as_ptr();
    }
    state.check_sleep_gap();
    if kind == CGEventType::TapDisabledByTimeout || kind == CGEventType::TapDisabledByUserInput {
        state.reset();
        return event.as_ptr();
    }
    if kind != CGEventType::KeyDown && kind != CGEventType::KeyUp {
        return event.as_ptr();
    }
    if state.suspended || state.sleeping || secure_input() {
        return event.as_ptr();
    }
    // SAFETY: Ordinary key callbacks supply a live event for this call only.
    let key_event = Some(unsafe { event.as_ref() });
    let Ok(code) = u16::try_from(CGEvent::integer_value_field(
        key_event,
        CGEventField::KeyboardEventKeycode,
    )) else {
        return event.as_ptr();
    };
    let repeat =
        CGEvent::integer_value_field(key_event, CGEventField::KeyboardEventAutorepeat) != 0;
    let shortcut = CGEvent::flags(key_event).intersects(
        CGEventFlags::MaskCommand | CGEventFlags::MaskControl | CGEventFlags::MaskAlternate,
    );
    if let Some(input) = state.keys.event(
        code,
        kind == CGEventType::KeyDown,
        repeat,
        mac_typing_key(code),
        shortcut,
    ) {
        state.sink.input(input);
        state.changed = true;
    }
    // ListenOnly cannot suppress or modify the event sent to the foreground app.
    event.as_ptr()
}

struct Tap {
    port: CFRetained<CFMachPort>,
    source: Option<CFRetained<CFRunLoopSource>>,
}

impl Drop for Tap {
    fn drop(&mut self) {
        CGEvent::tap_enable(&self.port, false);
        if let Some(source) = &self.source {
            source.invalidate();
        }
        self.port.invalidate();
    }
}

unsafe extern "C" fn on_power(
    context: *mut c_void,
    _service: u32,
    message: u32,
    token: *mut c_void,
) {
    // SAFETY: Power is dropped before the boxed context. Both notification
    // sources run serially on the same private CFRunLoop.
    let state = unsafe { &mut *context.cast::<Callback<'_>>() };
    if message == WILL_SLEEP || message == DID_WAKE {
        state.sleeping = message == WILL_SLEEP;
        state.reset();
    }
    if message == CAN_SLEEP || message == WILL_SLEEP {
        // SAFETY: IOKit supplied this token for the live connection. Always
        // acknowledge immediately; BongoDSH must never prevent system sleep.
        unsafe {
            IOAllowPowerChange(state.power_connection, token as isize);
        }
    }
}

struct Power {
    connection: u32,
    notifier: u32,
    port: *mut c_void,
}

impl Drop for Power {
    fn drop(&mut self) {
        // SAFETY: These handles belong to this registration and are closed
        // exactly once, before the callback context can be dropped. Destroying
        // the port also invalidates its borrowed CFRunLoopSource.
        unsafe {
            if self.notifier != 0 {
                IODeregisterForSystemPower(&mut self.notifier);
            }
            if self.connection != 0 {
                IOServiceClose(self.connection);
            }
            if !self.port.is_null() {
                IONotificationPortDestroy(self.port);
            }
        }
    }
}

pub(super) fn listen(sink: &Sink) -> Result<(), Status> {
    if !has_permission() {
        return Err(Status::PermissionRequired);
    }
    let run_loop = CFRunLoop::current()
        .ok_or_else(|| Status::Failed("Cannot create keyboard run loop".to_owned()))?;
    let mut state = Box::new(Callback {
        sink,
        keys: KeyFilter::default(),
        heartbeat: SystemTime::now(),
        changed: false,
        suspended: secure_input(),
        sleeping: false,
        power_connection: 0,
    });
    let mask = (1 << CGEventType::KeyDown.0) | (1 << CGEventType::KeyUp.0);
    // SAFETY: The callback reads only event metadata. Its boxed context outlives
    // Tap, whose Drop invalidates both the source and port before freeing it.
    let port = unsafe {
        CGEvent::tap_create(
            CGEventTapLocation::SessionEventTap,
            CGEventTapPlacement::TailAppendEventTap,
            CGEventTapOptions::ListenOnly,
            mask,
            Some(on_event),
            (&mut *state as *mut Callback<'_>).cast(),
        )
    }
    .ok_or_else(|| {
        if has_permission() {
            Status::Failed(
                "Cannot create keyboard listener; restart after granting Input Monitoring"
                    .to_owned(),
            )
        } else {
            Status::PermissionRequired
        }
    })?;
    let mut tap = Tap { port, source: None };
    tap.source = Some(
        CFMachPort::new_run_loop_source(None, Some(&tap.port), 0)
            .ok_or_else(|| Status::Failed("Cannot create keyboard event source".to_owned()))?,
    );
    // SAFETY: Core Foundation exports an immutable, process-lifetime mode value.
    let mode = unsafe { kCFRunLoopDefaultMode };
    let mut power = Power {
        connection: 0,
        notifier: 0,
        port: null_mut(),
    };
    // SAFETY: The output handles are writable. Power owns the registration and
    // drops before state; callbacks begin only when this run loop is pumped.
    power.connection = unsafe {
        IORegisterForSystemPower(
            (&mut *state as *mut Callback<'_>).cast(),
            &mut power.port,
            on_power,
            &mut power.notifier,
        )
    };
    if power.connection == 0 || power.port.is_null() {
        return Err(Status::Failed(
            "Cannot subscribe to system sleep notifications".to_owned(),
        ));
    }
    state.power_connection = power.connection;
    // SAFETY: A successful registration owns this port and its borrowed source.
    let power_source = unsafe { IONotificationPortGetRunLoopSource(power.port).as_ref() }
        .ok_or_else(|| Status::Failed("Cannot create system sleep event source".to_owned()))?;
    run_loop.add_source(Some(power_source), mode);
    run_loop.add_source(tap.source.as_deref(), mode);
    CGEvent::tap_enable(&tap.port, true);
    sink.status(if state.suspended {
        Status::Suspended
    } else {
        Status::Listening
    });
    let mut next_check = Instant::now();
    while !sink.stopped.load(Ordering::Acquire) {
        state.check_sleep_gap();
        let suspended = state.sleeping || secure_input();
        if state.suspended != suspended {
            state.reset();
            state.suspended = suspended;
            sink.status(if suspended {
                Status::Suspended
            } else {
                Status::Listening
            });
        }
        if Instant::now() >= next_check {
            next_check = Instant::now() + Duration::from_secs(1);
            if !has_permission() {
                return Err(Status::PermissionRequired);
            }
            if !tap.port.is_valid() {
                return Err(Status::Failed(
                    "Keyboard event source disconnected".to_owned(),
                ));
            }
            if !CGEvent::tap_is_enabled(&tap.port) && !state.suspended {
                state.reset();
                CGEvent::tap_enable(&tap.port, true);
                if !CGEvent::tap_is_enabled(&tap.port) {
                    return Err(Status::Failed(
                        "Keyboard listener disabled; reconnect from settings".to_owned(),
                    ));
                }
            }
            // Reconcile missing releases after device removal or secure input.
            let released: Vec<_> = state
                .keys
                .tracked()
                .filter(|code| {
                    !CGEventSource::key_state(CGEventSourceStateID::CombinedSessionState, *code)
                })
                .collect();
            for code in released {
                if let Some(input) = state.keys.event(code, false, false, true, false) {
                    sink.input(input);
                    state.changed = true;
                }
            }
        }
        if state.changed {
            state.changed = false;
            sink.wake();
        }
        // Repaint happens outside the callback. A bounded wait also lets Drop
        // join the listener without sending native pointers across threads.
        CFRunLoop::run_in_mode(mode, 0.25, true);
    }
    Ok(())
}
