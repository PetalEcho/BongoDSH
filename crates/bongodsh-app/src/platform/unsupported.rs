use eframe::egui::{Pos2, Rect};
use raw_window_handle::RawWindowHandle;

use crate::positioning::{Monitor, TargetWindow, Widget};

pub(crate) struct NativeWindow;

impl NativeWindow {
    pub fn new(_: RawWindowHandle) -> Option<Self> {
        None
    }
    pub fn geometry(&self, _: Rect) -> Option<(Widget, Vec<Monitor>)> {
        None
    }
    pub fn cursor(&self) -> Option<Pos2> {
        None
    }
    pub fn primary_down(&self) -> bool {
        false
    }
    pub fn move_to(&self, _: Pos2) {}
}

pub(crate) fn visible_windows() -> Vec<TargetWindow> {
    Vec::new()
}

pub(crate) fn has_focus_permission() -> bool {
    false
}
pub(crate) fn window_is_focused(_: crate::positioning::WindowId) -> bool {
    false
}
