use std::{
    ffi::c_void,
    time::{Duration, Instant},
};

use core_foundation::{
    array::CFArray,
    base::{CFType, CFTypeRef, TCFType},
    boolean::CFBoolean,
    dictionary::CFDictionary,
    number::CFNumber,
    string::{CFString, CFStringRef},
};
use core_graphics::{
    geometry::{CGPoint, CGRect, CGSize},
    window,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};
use objc2::{MainThreadMarker, rc::Retained, runtime::NSObjectProtocol, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSEvent, NSScreen, NSView, NSWindow,
    NSWindowCollectionBehavior,
};
use objc2_foundation::{NSPoint, NSProcessInfo, NSRect};
use raw_window_handle::RawWindowHandle;

use super::{Request, Snapshot};
use crate::positioning::{
    AttachmentKind, Monitor, TargetWindow, Widget, WindowId, covers_screen, input_distance,
    pick_window,
};

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

    pub fn geometry(&self) -> Option<(Widget, Vec<Monitor>)> {
        let screens = NSScreen::screens(self.main_thread);
        let origin_height = screens.firstObject()?.frame().size.height;
        let monitors = screens
            .iter()
            .map(|screen| {
                let bounds = desktop_rect(screen.frame(), origin_height);
                let mut fullscreen_area = bounds;
                if screen.respondsToSelector(sel!(safeAreaInsets)) {
                    let safe = screen.safeAreaInsets();
                    fullscreen_area.min += vec2(safe.left as f32, safe.top as f32);
                    fullscreen_area.max -= vec2(safe.right as f32, safe.bottom as f32);
                }
                Monitor {
                    bounds,
                    work_area: desktop_rect(screen.visibleFrame(), origin_height),
                    fullscreen_area,
                }
            })
            .collect();
        Some((
            Widget {
                outer: desktop_rect(self.window.frame(), origin_height),
                scale: 1.0,
            },
            monitors,
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

pub(crate) struct Probe {
    input: Option<(WindowId, Element)>,
}

impl Probe {
    pub fn new() -> Self {
        Self { input: None }
    }

    pub fn sample(&mut self, request: &Request) -> Snapshot {
        // SAFETY: this only reads the calling process's accessibility trust, without prompting.
        let input_access = unsafe { AXIsProcessTrusted() };
        let mut windows = visible_windows(&request.monitors);
        if input_access
            && let Some(front) = windows.first_mut()
            && let Some(element) = window_element(*front)
        {
            front.fullscreen |= element.boolean("AXFullScreen").unwrap_or(false);
        }
        if !request.inputs || !input_access {
            self.input = None;
        } else if request.snap {
            self.input = pick_window(request.widget, &windows).and_then(|host| {
                let root = window_element(*host)?;
                find_input(root, request.widget).map(|element| (host.id, element))
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
            let rect = element.bounds()?;
            (host.bounds.intersects(rect) && !element.boolean("AXHidden").unwrap_or(false))
                .then_some((*id, rect))
        });
        Snapshot {
            generation: request.generation,
            snap: request.snap,
            windows,
            input,
            input_access,
        }
    }
}

fn visible_windows(monitors: &[Monitor]) -> Vec<TargetWindow> {
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
                fullscreen: covers_screen(bounds, monitors),
            })
        })
        .collect()
}

struct Element(CFType);

impl Element {
    fn from_value(value: CFType) -> Option<Self> {
        // SAFETY: value owns a valid Core Foundation reference; GetTypeID has no arguments.
        (value.type_of() == unsafe { AXUIElementGetTypeID() }).then_some(Self(value))
    }

    fn application(process: u32) -> Option<Self> {
        let process = i32::try_from(process).ok()?;
        // SAFETY: the PID is a value obtained from the window server. Null is checked before wrapping.
        let reference = unsafe { AXUIElementCreateApplication(process) };
        if reference.is_null() {
            return None;
        }
        // SAFETY: Create returns an owned AXUIElement. The timeout applies to our client's requests.
        unsafe {
            AXUIElementSetMessagingTimeout(reference, 0.05);
            Some(Self(CFType::wrap_under_create_rule(reference)))
        }
    }

    fn attribute(&self, name: &str) -> Option<CFType> {
        let name = CFString::new(name);
        let mut value = std::ptr::null();
        // SAFETY: self and name retain valid objects; value points to initialized local storage.
        let result = unsafe {
            AXUIElementCopyAttributeValue(
                self.0.as_CFTypeRef(),
                name.as_concrete_TypeRef(),
                &mut value,
            )
        };
        if result != 0 || value.is_null() {
            return None;
        }
        // SAFETY: a successful Copy call transfers one owned reference to the caller.
        Some(unsafe { CFType::wrap_under_create_rule(value) })
    }

    fn boolean(&self, name: &str) -> Option<bool> {
        Some(self.attribute(name)?.downcast::<CFBoolean>()?.into())
    }

    fn elements(&self, name: &str) -> Vec<Self> {
        self.attribute(name)
            .and_then(|value| value.downcast::<CFArray>())
            .map(|array| {
                array
                    .iter()
                    .take(256)
                    .filter_map(|value| {
                        // SAFETY: AXWindows/AXChildren arrays contain CF objects; retain before the array is dropped.
                        let value = unsafe { CFType::wrap_under_get_rule(*value) };
                        Self::from_value(value)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn bounds(&self) -> Option<Rect> {
        let position = self.attribute("AXPosition")?;
        let size = self.attribute("AXSize")?;
        let mut point = CGPoint::new(0.0, 0.0);
        let mut dimensions = CGSize::new(0.0, 0.0);
        // SAFETY: type IDs and AXValue kinds are checked before writing into matching CGPoint/CGSize storage.
        let valid = unsafe {
            position.type_of() == AXValueGetTypeID()
                && size.type_of() == AXValueGetTypeID()
                && AXValueGetType(position.as_CFTypeRef()) == 1
                && AXValueGetType(size.as_CFTypeRef()) == 2
                && AXValueGetValue(
                    position.as_CFTypeRef(),
                    1,
                    (&mut point as *mut CGPoint).cast(),
                )
                && AXValueGetValue(
                    size.as_CFTypeRef(),
                    2,
                    (&mut dimensions as *mut CGSize).cast(),
                )
        };
        let rect = Rect::from_min_size(
            pos2(point.x as f32, point.y as f32),
            vec2(dimensions.width as f32, dimensions.height as f32),
        );
        (valid && rect.is_finite() && rect.is_positive()).then_some(rect)
    }

    fn is_input(&self) -> bool {
        let Some(role) = self
            .attribute("AXRole")
            .and_then(|value| value.downcast::<CFString>())
        else {
            return false;
        };
        matches!(
            role.to_string().as_str(),
            "AXTextField" | "AXTextArea" | "AXComboBox"
        ) && self.boolean("AXEnabled").unwrap_or(true)
            && !self.boolean("AXHidden").unwrap_or(false)
    }
}

fn window_element(window: TargetWindow) -> Option<Element> {
    Element::application(window.id.process)?
        .elements("AXWindows")
        .into_iter()
        .find(|element| {
            element.bounds().is_some_and(|bounds| {
                bounds.min.distance(window.bounds.min) < 4.0
                    && bounds.max.distance(window.bounds.max) < 4.0
            })
        })
}

fn find_input(root: Element, widget: Widget) -> Option<Element> {
    let deadline = Instant::now() + Duration::from_millis(180);
    let mut stack = vec![root];
    let mut best: Option<(f32, Element)> = None;
    // A bounded walk also handles web accessibility trees without reading text or values.
    for _ in 0..256 {
        if Instant::now() >= deadline {
            break;
        }
        let Some(element) = stack.pop() else { break };
        if let Some(bounds) = element.bounds() {
            if !bounds
                .expand(48.0 * widget.scale)
                .intersects(widget.content())
            {
                continue;
            }
            if element.is_input()
                && let Some(distance) = input_distance(widget, bounds)
                && best
                    .as_ref()
                    .is_none_or(|(previous, _)| distance < *previous)
            {
                best = Some((distance, element));
                continue;
            }
        }
        let remaining = 256usize.saturating_sub(stack.len());
        stack.extend(element.elements("AXChildren").into_iter().take(remaining));
    }
    best.map(|(_, element)| element)
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementGetTypeID() -> usize;
    fn AXUIElementCreateApplication(process: i32) -> CFTypeRef;
    fn AXUIElementSetMessagingTimeout(element: CFTypeRef, timeout: f32) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: CFTypeRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
    fn AXValueGetTypeID() -> usize;
    fn AXValueGetType(value: CFTypeRef) -> u32;
    fn AXValueGetValue(value: CFTypeRef, kind: u32, output: *mut c_void) -> bool;
}
