#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]

use core_foundation::{
    base::{CFType, TCFType},
    dictionary::CFDictionary,
    number::CFNumber,
    string::CFString,
};
use core_graphics::{geometry::CGRect, window};
use eframe::egui::{Pos2, Rect, pos2, vec2};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSEvent, NSScreen, NSView, NSWindow,
    NSWindowCollectionBehavior,
};
use objc2_foundation::{NSPoint, NSProcessInfo, NSRect};
use raw_window_handle::RawWindowHandle;

use crate::positioning::{Monitor, TargetWindow, Widget, WindowId};

pub(crate) struct NativeWindow {
    window: Retained<NSWindow>,
    main_thread: MainThreadMarker,
}

impl NativeWindow {
    pub fn new(handle: RawWindowHandle) -> Option<Self> {
        let RawWindowHandle::AppKit(handle) = handle else {
            return None;
        };
        let main_thread = MainThreadMarker::new()?;
        // SAFETY: eframe lends a live NSView handle on its UI thread. Retain its window before returning.
        let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
        let window = view.window()?;
        // An accessory app can accompany another app's fullscreen Space without adding a Dock icon.
        NSApplication::sharedApplication(main_thread)
            .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        let mut behavior = window.collectionBehavior();
        behavior.remove(
            NSWindowCollectionBehavior::MoveToActiveSpace
                | NSWindowCollectionBehavior::FullScreenPrimary
                | NSWindowCollectionBehavior::FullScreenNone,
        );
        behavior.insert(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        if NSProcessInfo::processInfo()
            .operatingSystemVersion()
            .majorVersion
            >= 13
        {
            behavior.remove(
                NSWindowCollectionBehavior::Primary | NSWindowCollectionBehavior::Auxiliary,
            );
            behavior.insert(NSWindowCollectionBehavior::CanJoinAllApplications);
        }
        window.setCollectionBehavior(behavior);
        Some(Self {
            window,
            main_thread,
        })
    }

    pub fn geometry(&self, content: Rect) -> Option<(Widget, Vec<Monitor>)> {
        let screens = NSScreen::screens(self.main_thread);
        let origin_height = screens.firstObject()?.frame().size.height;
        let monitors = screens
            .iter()
            .map(|screen| {
                let bounds = desktop_rect(screen.frame(), origin_height);
                Monitor {
                    bounds,
                    work_area: desktop_rect(screen.visibleFrame(), origin_height),
                }
            })
            .collect();
        Some((
            Widget {
                outer: desktop_rect(self.window.frame(), origin_height),
                content: content.translate(
                    desktop_rect(self.window.frame(), origin_height)
                        .min
                        .to_vec2(),
                ),
                scale: 1.0,
            },
            monitors,
        ))
    }

    pub fn cursor(&self) -> Option<Pos2> {
        let primary = NSScreen::screens(self.main_thread).firstObject()?;
        let point = NSEvent::mouseLocation();
        Some(pos2(
            point.x as f32,
            (primary.frame().size.height - point.y) as f32,
        ))
    }

    pub fn primary_down(&self) -> bool {
        NSEvent::pressedMouseButtons() & 1 != 0
    }

    pub fn move_to(&self, position: Pos2) {
        let screens = NSScreen::screens(self.main_thread);
        if let Some(primary) = screens.firstObject() {
            self.window.setFrameTopLeftPoint(NSPoint::new(
                f64::from(position.x),
                primary.frame().size.height - f64::from(position.y),
            ));
        }
    }
}

fn desktop_rect(rect: NSRect, primary_height: f64) -> Rect {
    Rect::from_min_size(
        pos2(
            rect.origin.x as f32,
            (primary_height - rect.origin.y - rect.size.height) as f32,
        ),
        vec2(rect.size.width as f32, rect.size.height as f32),
    )
}

pub fn visible_windows() -> Vec<TargetWindow> {
    let Some(list) = window::copy_window_info(
        window::kCGWindowListOptionOnScreenOnly | window::kCGWindowListExcludeDesktopElements,
        window::kCGNullWindowID,
    ) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|entry| {
            // SAFETY: CGWindowListCopyWindowInfo returns a CFArray of retained CFDictionary objects.
            let dictionary =
                unsafe { CFDictionary::<CFString, CFType>::wrap_under_get_rule((*entry).cast()) };
            let number = |key: &str| {
                dictionary
                    .find(CFString::new(key))?
                    .downcast::<CFNumber>()?
                    .to_i64()
            };
            let process = u32::try_from(number("kCGWindowOwnerPID")?).ok()?;
            if process == std::process::id() || number("kCGWindowLayer")? != 0 {
                return None;
            }
            let dictionary_bounds = dictionary
                .find(CFString::new("kCGWindowBounds"))?
                .downcast::<CFDictionary>()?;
            let bounds = CGRect::from_dict_representation(&dictionary_bounds)?;
            let bounds = Rect::from_min_size(
                pos2(bounds.origin.x as f32, bounds.origin.y as f32),
                vec2(bounds.size.width as f32, bounds.size.height as f32),
            );
            if !bounds.is_finite() || bounds.width() < 80.0 || bounds.height() < 50.0 {
                return None;
            }
            Some(TargetWindow {
                id: WindowId {
                    handle: u64::try_from(number("kCGWindowNumber")?).ok()?,
                    process,
                },
                bounds,
                maximized: false,
            })
        })
        .collect()
}

// SAFETY: Signatures and AXValue type tags follow the public HIServices headers.
// CFTypeRef is used for opaque AX objects, with runtime type checks before use.
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> u8;
    fn AXUIElementCreateSystemWide() -> core_foundation::base::CFTypeRef;
    fn AXUIElementGetTypeID() -> core_foundation::base::CFTypeID;
    fn AXUIElementSetMessagingTimeout(
        element: core_foundation::base::CFTypeRef,
        timeout: f32,
    ) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: core_foundation::base::CFTypeRef,
        attribute: core_foundation::string::CFStringRef,
        value: *mut core_foundation::base::CFTypeRef,
    ) -> i32;
    fn AXUIElementGetPid(element: core_foundation::base::CFTypeRef, pid: *mut i32) -> i32;
    fn AXValueGetTypeID() -> core_foundation::base::CFTypeID;
    fn AXValueGetValue(
        value: core_foundation::base::CFTypeRef,
        kind: u32,
        output: *mut std::ffi::c_void,
    ) -> u8;
}

pub(crate) fn has_focus_permission() -> bool {
    // SAFETY: Read-only permission preflight, no pointers and no system prompt.
    unsafe { AXIsProcessTrusted() != 0 }
}

pub(crate) fn request_focus_permission() -> bool {
    let options = CFDictionary::from_CFType_pairs(&[(
        CFString::new("AXTrustedCheckOptionPrompt").as_CFType(),
        core_foundation::boolean::CFBoolean::true_value().as_CFType(),
    )]);
    // SAFETY: This valid options dictionary lives through the synchronous call.
    // Called only by an explicit UI authorization action.
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) != 0 }
}

pub(crate) fn open_focus_settings() -> Result<(), String> {
    std::process::Command::new("/usr/bin/open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()
        .map_err(|error| error.to_string())?
        .success()
        .then_some(())
        .ok_or_else(|| "Cannot open Accessibility settings".to_owned())
}

fn focus_attribute(element: &CFType, name: &str) -> Option<CFType> {
    let attribute = CFString::new(name);
    let mut value = std::ptr::null();
    // SAFETY: Runtime check ensures an AX element, and the output is writable.
    // Copy returns an owned CF object, released by the wrapper on every path.
    unsafe {
        if element.type_of() != AXUIElementGetTypeID() {
            return None;
        }
        let result = AXUIElementCopyAttributeValue(
            element.as_CFTypeRef(),
            attribute.as_concrete_TypeRef(),
            &mut value,
        );
        let value = (!value.is_null()).then(|| CFType::wrap_under_create_rule(value));
        (result == 0).then_some(value).flatten()
    }
}

fn focused_geometry() -> Option<(u32, Rect)> {
    use core_graphics::geometry::{CGPoint, CGSize};
    // SAFETY: Create returns an owned AX system element, checked for null. The
    // process-wide 5 ms IPC timeout bounds queries to unresponsive applications.
    let system = unsafe {
        let raw = AXUIElementCreateSystemWide();
        if raw.is_null() {
            return None;
        }
        let system = CFType::wrap_under_create_rule(raw);
        if AXUIElementSetMessagingTimeout(raw, 0.005) != 0 {
            return None;
        }
        system
    };
    let app = focus_attribute(&system, "AXFocusedApplication")?;
    let mut pid = 0;
    // SAFETY: Verify the returned object is an AX element before querying its PID.
    unsafe {
        if app.type_of() != AXUIElementGetTypeID()
            || AXUIElementGetPid(app.as_CFTypeRef(), &mut pid) != 0
        {
            return None;
        }
    }
    let window = focus_attribute(&app, "AXFocusedWindow")?;
    let position = focus_attribute(&window, "AXPosition")?;
    let size = focus_attribute(&window, "AXSize")?;
    let mut point = CGPoint::new(0.0, 0.0);
    let mut extent = CGSize::new(0.0, 0.0);
    // SAFETY: Both values are checked as AXValue. Tags 1/2 request CGPoint/CGSize
    // and outputs point to correctly sized, aligned local structs. No text is read.
    unsafe {
        if position.type_of() != AXValueGetTypeID()
            || size.type_of() != AXValueGetTypeID()
            || AXValueGetValue(
                position.as_CFTypeRef(),
                1,
                (&mut point as *mut CGPoint).cast(),
            ) == 0
            || AXValueGetValue(size.as_CFTypeRef(), 2, (&mut extent as *mut CGSize).cast()) == 0
        {
            return None;
        }
    }
    let bounds = Rect::from_min_size(
        pos2(point.x as f32, point.y as f32),
        vec2(extent.width as f32, extent.height as f32),
    );
    bounds
        .is_finite()
        .then_some((u32::try_from(pid).ok()?, bounds))
}

pub(crate) fn window_is_focused(target: WindowId) -> bool {
    if !has_focus_permission() {
        return false;
    }
    let Some((process, bounds)) = focused_geometry() else {
        return false;
    };
    if process != target.process {
        return false;
    }
    // Public AX APIs expose geometry, not CGWindowID. Require a unique match;
    // overlapping identical windows or unsupported AX providers fail closed.
    crate::positioning::identify_focused_window(process, bounds, &visible_windows()) == Some(target)
}
