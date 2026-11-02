//! Placement uses native desktop coordinates: points on macOS, pixels on Windows.

use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WindowId {
    pub handle: u64,
    pub process: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TargetWindow {
    pub id: WindowId,
    pub bounds: Rect,
    pub fullscreen: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Monitor {
    pub bounds: Rect,
    pub work_area: Rect,
    /// Fullscreen content can exclude a camera housing while still covering the display.
    pub fullscreen_area: Rect,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Widget {
    pub outer: Rect,
    pub scale: f32,
}

impl Widget {
    pub fn content(self) -> Rect {
        Rect::from_min_max(
            self.outer.min + vec2(53.0, 35.0) * self.scale,
            self.outer.min + vec2(269.0, 219.0) * self.scale,
        )
    }

    fn outer_position(self, content: Pos2) -> Pos2 {
        content - (self.content().min - self.outer.min)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttachmentKind {
    Window,
    Input,
}

#[derive(Clone, Copy, Debug)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
struct Anchor {
    edge: Edge,
    fraction: f32,
    inside: bool,
}

impl Anchor {
    fn position(self, target: Rect, size: Vec2, gap: f32) -> Pos2 {
        let x = target.left() + (target.width() - size.x).max(0.0) * self.fraction;
        let y = target.top() + (target.height() - size.y).max(0.0) * self.fraction;
        match (self.edge, self.inside) {
            (Edge::Top, false) => pos2(x, target.top() - size.y - gap),
            (Edge::Top, true) => pos2(x, target.top() + gap),
            (Edge::Bottom, false) => pos2(x, target.bottom() + gap),
            (Edge::Bottom, true) => pos2(x, target.bottom() - size.y - gap),
            (Edge::Left, false) => pos2(target.left() - size.x - gap, y),
            (Edge::Left, true) => pos2(target.left() + gap, y),
            (Edge::Right, false) => pos2(target.right() + gap, y),
            (Edge::Right, true) => pos2(target.right() - size.x - gap, y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Attachment {
    window: WindowId,
    kind: AttachmentKind,
    anchor: Anchor,
}

#[derive(Default)]
pub(crate) struct Placement {
    attachment: Option<Attachment>,
    fullscreen: Option<(WindowId, Pos2)>,
    manual_fullscreen: Option<WindowId>,
}

impl Placement {
    pub fn target(&self) -> Option<(WindowId, AttachmentKind)> {
        self.attachment.map(|target| (target.window, target.kind))
    }

    pub fn in_fullscreen(&self) -> bool {
        self.fullscreen.is_some()
    }

    pub fn detach(&mut self) {
        self.attachment = None;
        if let Some((window, _)) = self.fullscreen.take() {
            self.manual_fullscreen = Some(window);
        }
    }

    pub fn snap(
        &mut self,
        widget: Widget,
        windows: &[TargetWindow],
        input: Option<(WindowId, Rect)>,
        monitors: &[Monitor],
    ) -> Option<Pos2> {
        self.detach();
        self.manual_fullscreen = windows.first().filter(|w| w.fullscreen).map(|w| w.id);
        let host = pick_window(widget, windows)?;
        let monitor = monitor_for(host.bounds.center(), monitors)?;
        if let Some((id, bounds)) = input.filter(|(id, _)| *id == host.id)
            && let Some(anchor) = input_anchor(widget, bounds, monitor.work_area)
        {
            self.attachment = Some(Attachment {
                window: id,
                kind: AttachmentKind::Input,
                anchor,
            });
            return Some(self.at_anchor(widget, bounds, anchor, monitor.work_area));
        }
        let anchor = window_anchor(widget, host.bounds)?;
        self.attachment = Some(Attachment {
            window: host.id,
            kind: AttachmentKind::Window,
            anchor,
        });
        Some(self.at_anchor(widget, host.bounds, anchor, monitor.work_area))
    }

    pub fn update(
        &mut self,
        widget: Widget,
        windows: &[TargetWindow],
        input: Option<(WindowId, Rect)>,
        monitors: &[Monitor],
        fullscreen_enabled: bool,
    ) -> Option<Pos2> {
        let foreground = windows.first().filter(|window| window.fullscreen);
        if foreground.map(|w| w.id) != self.manual_fullscreen {
            self.manual_fullscreen = None;
        }
        if fullscreen_enabled
            && let Some(window) = foreground
            && self.manual_fullscreen != Some(window.id)
        {
            let restore = self.fullscreen.map_or(widget.outer.min, |(_, saved)| saved);
            self.fullscreen = Some((window.id, restore));
            let monitor = monitor_for(window.bounds.center(), monitors)?;
            return Some(bottom_right(widget, monitor.work_area));
        }
        let restored = self.fullscreen.take().map(|(_, position)| position);
        if let Some(attachment) = self.attachment {
            if let Some(window) = windows.iter().find(|window| window.id == attachment.window) {
                let bounds = match attachment.kind {
                    AttachmentKind::Window => Some(window.bounds),
                    AttachmentKind::Input => input
                        .filter(|(id, bounds)| {
                            *id == window.id && window.bounds.intersects(*bounds)
                        })
                        .map(|(_, bounds)| bounds),
                };
                if let Some(bounds) = bounds {
                    let monitor = monitor_for(window.bounds.center(), monitors)?;
                    let anchor = match attachment.kind {
                        AttachmentKind::Window => Some(attachment.anchor),
                        AttachmentKind::Input => safe_input_anchor(
                            widget,
                            bounds,
                            attachment.anchor.fraction,
                            monitor.work_area,
                        ),
                    };
                    if let Some(anchor) = anchor {
                        return Some(self.at_anchor(widget, bounds, anchor, monitor.work_area));
                    }
                }
            }
            // Closed, minimized, hidden or inaccessible targets must not leave a stale attachment.
            self.attachment = None;
        }
        let position = restored.unwrap_or(widget.outer.min);
        let monitor = monitor_for(position + widget.outer.size() * 0.5, monitors)?;
        Some(clamp_position(
            position,
            widget.outer.size(),
            monitor.work_area,
        ))
    }

    fn at_anchor(&self, widget: Widget, bounds: Rect, anchor: Anchor, area: Rect) -> Pos2 {
        let position = anchor.position(bounds, widget.content().size(), 8.0 * widget.scale);
        clamp_position(widget.outer_position(position), widget.outer.size(), area)
    }
}

pub(crate) fn monitor_for(point: Pos2, monitors: &[Monitor]) -> Option<&Monitor> {
    monitors.iter().min_by(|a, b| {
        a.bounds
            .distance_sq_to_pos(point)
            .total_cmp(&b.bounds.distance_sq_to_pos(point))
    })
}

pub(crate) fn clamp_position(position: Pos2, size: Vec2, area: Rect) -> Pos2 {
    pos2(
        position
            .x
            .clamp(area.left(), (area.right() - size.x).max(area.left())),
        position
            .y
            .clamp(area.top(), (area.bottom() - size.y).max(area.top())),
    )
}

pub(crate) fn bottom_right(widget: Widget, area: Rect) -> Pos2 {
    clamp_position(
        area.max - widget.outer.size() - Vec2::splat(24.0 * widget.scale),
        widget.outer.size(),
        area,
    )
}

pub(crate) fn covers_screen(bounds: Rect, monitors: &[Monitor]) -> bool {
    monitors
        .iter()
        .any(|screen| bounds.expand(2.0).contains_rect(screen.fullscreen_area))
}

pub(crate) fn pick_window(widget: Widget, windows: &[TargetWindow]) -> Option<&TargetWindow> {
    let content = widget.content();
    // Respect z-order: never attach to an obscured window behind the one being pointed at.
    windows.iter().find(|window| {
        window
            .bounds
            .expand(32.0 * widget.scale)
            .intersects(content)
    })
}

pub(crate) fn input_distance(widget: Widget, bounds: Rect) -> Option<f32> {
    if !bounds.is_finite() || bounds.width() < 8.0 || bounds.height() < 8.0 {
        return None;
    }
    let content = widget.content();
    let horizontal = content.right() >= bounds.left() && content.left() <= bounds.right();
    let distance = (content.bottom() - bounds.top())
        .abs()
        .min((content.top() - bounds.bottom()).abs());
    (horizontal && (distance <= 48.0 * widget.scale || content.intersects(bounds)))
        .then_some(distance)
}

fn input_anchor(widget: Widget, bounds: Rect, area: Rect) -> Option<Anchor> {
    input_distance(widget, bounds)?;
    let fraction = fraction(
        widget.content().left(),
        bounds.left(),
        bounds.width() - widget.content().width(),
    );
    safe_input_anchor(widget, bounds, fraction, area)
}

fn safe_input_anchor(widget: Widget, bounds: Rect, fraction: f32, area: Rect) -> Option<Anchor> {
    [Edge::Top, Edge::Bottom, Edge::Right, Edge::Left]
        .into_iter()
        .find_map(|edge| {
            let anchor = Anchor {
                edge,
                fraction,
                inside: false,
            };
            let position = widget.outer_position(anchor.position(
                bounds,
                widget.content().size(),
                8.0 * widget.scale,
            ));
            // Do not clamp onto the input text when there is no room above it.
            area.contains_rect(Rect::from_min_size(position, widget.outer.size()))
                .then_some(anchor)
        })
}

fn window_anchor(widget: Widget, bounds: Rect) -> Option<Anchor> {
    let content = widget.content();
    [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right]
        .into_iter()
        .flat_map(|edge| {
            [false, true].map(|inside| Anchor {
                edge,
                inside,
                fraction: match edge {
                    Edge::Top | Edge::Bottom => fraction(
                        content.left(),
                        bounds.left(),
                        bounds.width() - content.width(),
                    ),
                    Edge::Left | Edge::Right => fraction(
                        content.top(),
                        bounds.top(),
                        bounds.height() - content.height(),
                    ),
                },
            })
        })
        .map(|anchor| {
            let position = anchor.position(bounds, content.size(), 8.0 * widget.scale);
            (anchor, position.distance(content.min))
        })
        .filter(|(_, distance)| *distance <= 32.0 * widget.scale)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(anchor, _)| anchor)
}

fn fraction(value: f32, start: f32, span: f32) -> f32 {
    if span > 0.0 {
        ((value - start) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
