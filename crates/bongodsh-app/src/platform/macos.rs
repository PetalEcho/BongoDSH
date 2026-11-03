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
            })
        })
        .collect()
}
