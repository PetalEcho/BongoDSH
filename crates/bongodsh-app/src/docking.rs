use std::time::{Duration, Instant};

use eframe::egui;
use raw_window_handle::HasWindowHandle;

use crate::{
    platform::{NativeWindow, Observer, Request},
    positioning::{AttachmentKind, Placement, bottom_right, monitor_for},
};

pub(crate) struct Docking {
    native: Option<NativeWindow>,
    observer: Option<Observer>,
    placement: Placement,
    enabled: bool,
    inputs: bool,
    fullscreen: bool,
    input_access: bool,
    dragging: bool,
    needs_snap: bool,
    awaiting: bool,
    generation: u64,
    next_check: Instant,
}

impl Docking {
    pub fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let native = creation
            .window_handle()
            .ok()
            .and_then(|handle| NativeWindow::new(handle.as_raw()));
        let observer =
            native
                .as_ref()
                .and_then(|_| match Observer::new(creation.egui_ctx.clone()) {
                    Ok(observer) => Some(observer),
                    Err(error) => {
                        eprintln!("无法启动窗口定位线程：{error}");
                        None
                    }
                });
        Self {
            native,
            observer,
            placement: Placement::default(),
            enabled: true,
            inputs: true,
            fullscreen: true,
            input_access: false,
            dragging: false,
            needs_snap: false,
            awaiting: false,
            generation: 0,
            next_check: Instant::now(),
        }
    }

    pub fn begin_drag(&mut self) {
        self.invalidate();
        self.placement.detach();
        self.dragging = true;
        self.needs_snap = false;
    }

    fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.next_check = Instant::now();
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let (Some(native), Some(observer)) = (&self.native, &self.observer) else {
            return;
        };
        let Some((widget, monitors)) = native.geometry() else {
            return;
        };
        let now = Instant::now();
        if self.dragging && !native.primary_down() {
            self.dragging = false;
            self.needs_snap = self.enabled;
            self.generation = self.generation.wrapping_add(1);
            self.next_check = now;
        }
        while let Some(snapshot) = observer.receive() {
            self.awaiting = false;
            if self.dragging || snapshot.generation != self.generation {
                continue;
            }
            self.input_access = snapshot.input_access;
            let position = if snapshot.snap && self.needs_snap {
                self.needs_snap = false;
                self.placement
                    .snap(widget, &snapshot.windows, snapshot.input, &monitors)
            } else {
                self.placement.update(
                    widget,
                    &snapshot.windows,
                    snapshot.input,
                    &monitors,
                    self.fullscreen,
                )
            };
            if let Some(position) = position
                && position.distance(widget.outer.min) > 0.5
            {
                native.move_to(position);
            }
            self.next_check = now
                + if self.placement.target().is_some() || self.placement.in_fullscreen() {
                    Duration::from_millis(200)
                } else {
                    Duration::from_millis(750)
                };
        }
        if self.dragging {
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        if !self.enabled && !self.fullscreen {
            return;
        }
        if !self.awaiting && now >= self.next_check {
            self.awaiting = observer.request(Request {
                generation: self.generation,
                widget,
                monitors,
                snap: self.needs_snap,
                inputs: self.inputs && self.enabled,
                follow: self.placement.target(),
            });
            if !self.awaiting {
                self.next_check = now + Duration::from_secs(1);
            }
        }
        if !self.awaiting {
            ctx.request_repaint_after(self.next_check.saturating_duration_since(now));
        }
    }

    pub fn status(&self, chinese: bool) -> &'static str {
        let text = |zh, en| if chinese { zh } else { en };
        if self.native.is_none() || self.observer.is_none() {
            text(
                "窗口定位不可用，可自由拖动",
                "Positioning unavailable; drag freely",
            )
        } else if self.dragging || self.needs_snap {
            text("松开后吸附附近目标", "Release near a window or input")
        } else if self.placement.in_fullscreen() {
            text("全屏 · 右下角", "Fullscreen · bottom right")
        } else {
            match self.placement.target().map(|(_, kind)| kind) {
                Some(AttachmentKind::Input) => text(
                    "已吸附输入框 · 拖动解除",
                    "Attached to input · drag to detach",
                ),
                Some(AttachmentKind::Window) => text(
                    "已吸附窗口 · 拖动解除",
                    "Attached to window · drag to detach",
                ),
                None => text(
                    "拖近窗口边缘或输入框后松开",
                    "Drop near a window edge or input",
                ),
            }
        }
    }

    pub fn menu(&mut self, ui: &mut egui::Ui, chinese: bool) {
        let text = |zh, en| if chinese { zh } else { en };
        ui.separator();
        ui.label(self.status(chinese));
        let available = self.native.is_some() && self.observer.is_some();
        ui.add_enabled_ui(available, |ui| {
            if ui
                .checkbox(
                    &mut self.enabled,
                    text("拖动吸附窗口", "Snap to windows on drop"),
                )
                .changed()
            {
                self.placement.detach();
                self.needs_snap = false;
                self.invalidate();
            }
            if ui
                .add_enabled(
                    self.enabled,
                    egui::Checkbox::new(
                        &mut self.inputs,
                        text("同时识别输入框", "Also snap to input fields"),
                    ),
                )
                .changed()
            {
                if self
                    .placement
                    .target()
                    .is_some_and(|(_, kind)| kind == AttachmentKind::Input)
                {
                    self.placement.detach();
                }
                self.invalidate();
            }
            if self.inputs && !self.input_access {
                #[cfg(target_os = "macos")]
                ui.small(text(
                    "输入框定位需在系统设置中授予辅助功能权限",
                    "Allow Accessibility in System Settings for input fields",
                ));
                #[cfg(target_os = "windows")]
                ui.small(text(
                    "输入框定位暂不可用，将使用窗口吸附",
                    "Input fields unavailable; window snapping still works",
                ));
            }
            if ui
                .checkbox(
                    &mut self.fullscreen,
                    text("全屏时移到右下角", "Bottom right in fullscreen"),
                )
                .changed()
            {
                self.invalidate();
            }
            if ui.button(text("解除吸附", "Detach")).clicked() {
                self.placement.detach();
                self.needs_snap = false;
                self.invalidate();
                ui.close();
            }
            if ui
                .button(text("移到当前屏幕右下角", "Move to bottom right"))
                .clicked()
            {
                if let Some(native) = &self.native
                    && let Some((widget, monitors)) = native.geometry()
                    && let Some(monitor) = monitor_for(widget.outer.center(), &monitors)
                {
                    native.move_to(bottom_right(widget, monitor.work_area));
                    self.placement.detach();
                    self.needs_snap = false;
                    self.invalidate();
                }
                ui.close();
            }
        });
    }
}
