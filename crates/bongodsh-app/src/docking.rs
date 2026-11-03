use std::time::{Duration, Instant};

use eframe::egui::{self, Pos2, Rect, Vec2};
use raw_window_handle::HasWindowHandle;

use crate::{
    platform::{NativeWindow, Observer},
    positioning::{Placement, clamp_to_monitors},
};

pub(crate) struct Docking {
    native: Option<NativeWindow>,
    observer: Option<Observer>,
    placement: Placement,
    initial_grab: Option<Vec2>,
    initial_position: Option<Pos2>,
    restore_requested: bool,
    following_pointer: Option<Vec2>,
    dragging: bool,
    enabled: bool,
    needs_snap: bool,
    awaiting: bool,
    generation: u64,
    next_check: Instant,
}

impl Docking {
    pub fn new(creation: &eframe::CreationContext<'_>, initial_grab: Option<Vec2>) -> Self {
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
                        eprintln!("无法启动窗口定位：{error}");
                        None
                    }
                });
        Self {
            native,
            observer,
            placement: Placement::default(),
            initial_grab,
            initial_position: None,
            restore_requested: false,
            following_pointer: None,
            dragging: false,
            enabled: true,
            needs_snap: false,
            awaiting: false,
            generation: 0,
            next_check: Instant::now(),
        }
    }

    pub fn available(&self) -> bool {
        self.native.is_some() && self.observer.is_some()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.placement.detach();
        self.needs_snap = enabled;
        self.invalidate_geometry();
    }

    pub fn invalidate_geometry(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.next_check = Instant::now();
    }

    pub fn restore_initial(&mut self) {
        self.restore_requested = self.initial_position.is_some();
        self.following_pointer = None;
        self.dragging = false;
        self.set_enabled(self.available());
    }

    pub fn panel_geometry(&self, desired: Vec2) -> Option<(Pos2, Vec2)> {
        let (widget, monitors) = self.native.as_ref()?.geometry(Rect::ZERO)?;
        let monitor = monitors.iter().min_by(|a, b| {
            a.bounds
                .distance_sq_to_pos(widget.outer.center())
                .total_cmp(&b.bounds.distance_sq_to_pos(widget.outer.center()))
        })?;
        let margin = 8.0 * widget.scale;
        let area = monitor.work_area.shrink(margin);
        let size = (desired * widget.scale).min(area.size());
        let above = widget.outer.top() - size.y - margin;
        let y = if above >= area.top() {
            above
        } else {
            widget.outer.bottom() + margin
        };
        let position = Pos2::new(
            (widget.outer.right() - size.x)
                .clamp(area.left(), (area.right() - size.x).max(area.left())),
            y.clamp(area.top(), (area.bottom() - size.y).max(area.top())),
        );
        Some((position / widget.scale, size / widget.scale))
    }

    pub fn begin_drag(&mut self) {
        self.placement.detach();
        self.following_pointer = None;
        self.dragging = true;
        self.needs_snap = false;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn update(&mut self, ctx: &egui::Context, content: Rect) {
        let (Some(native), Some(observer)) = (&self.native, &self.observer) else {
            return;
        };
        let Some((mut widget, monitors)) = native.geometry(content) else {
            return;
        };
        let now = Instant::now();
        if self.restore_requested {
            self.restore_requested = false;
            if let Some(position) = self.initial_position {
                let position = clamp_to_monitors(position, widget.outer.size(), &monitors);
                native.move_to(position);
                let delta = position - widget.outer.min;
                widget.outer = widget.outer.translate(delta);
                widget.content = widget.content.translate(delta);
            }
        }
        if let Some(grab) = self.initial_grab.take() {
            self.following_pointer = Some(grab);
            self.dragging = true;
        }
        if let Some(grab) = self.following_pointer
            && let Some(cursor) = native.cursor()
        {
            let position = clamp_to_monitors(
                cursor - widget.outer.size() * grab,
                widget.outer.size(),
                &monitors,
            );
            native.move_to(position);
            let delta = position - widget.outer.min;
            widget.outer = widget.outer.translate(delta);
            widget.content = widget.content.translate(delta);
        }
        if self.dragging && !native.primary_down() {
            if self.initial_position.is_none() {
                self.initial_position = Some(widget.outer.min);
            }
            self.dragging = false;
            self.following_pointer = None;
            self.needs_snap = self.enabled;
            self.generation = self.generation.wrapping_add(1);
            self.next_check = now;
        }
        while let Some(snapshot) = observer.receive() {
            self.awaiting = false;
            if self.dragging || snapshot.generation != self.generation {
                continue;
            }
            let position = if self.needs_snap {
                self.needs_snap = false;
                self.placement.snap(widget, &snapshot.windows)
            } else {
                self.placement.update(widget, &snapshot.windows)
            };
            let position = clamp_to_monitors(
                position.unwrap_or(widget.outer.min),
                widget.outer.size(),
                &monitors,
            );
            if position.distance(widget.outer.min) > 0.5 {
                native.move_to(position);
            }
            self.next_check = now + Duration::from_millis(200);
        }
        if self.dragging {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.enabled && (self.needs_snap || self.placement.attached()) {
            if !self.awaiting && now >= self.next_check {
                self.awaiting = observer.request(self.generation);
                if !self.awaiting {
                    self.next_check = now + Duration::from_secs(1);
                }
            }
            if !self.awaiting {
                ctx.request_repaint_after(self.next_check.saturating_duration_since(now));
            }
        }
    }

    pub fn menu(&mut self, ui: &mut egui::Ui, chinese: bool) {
        let text = |zh, en| if chinese { zh } else { en };
        ui.separator();
        ui.label(if self.placement.attached() {
            text(
                "已吸附窗口 · 拖动解除",
                "Attached to window; drag to detach",
            )
        } else {
            text("拖近其他窗口边缘后松开", "Drop near another window edge")
        });
        ui.add_enabled_ui(self.available(), |ui| {
            if ui
                .checkbox(
                    &mut self.enabled,
                    text("吸附其他程序窗口", "Snap to other app windows"),
                )
                .changed()
            {
                self.placement.detach();
                self.needs_snap = false;
                self.generation = self.generation.wrapping_add(1);
            }
            if self.placement.attached()
                && ui.button(text("解除吸附", "Detach from window")).clicked()
            {
                self.placement.detach();
                self.needs_snap = false;
                self.generation = self.generation.wrapping_add(1);
                ui.close();
            }
        });
    }
}
