//! Native coordinates are points on macOS and physical pixels on Windows.

use eframe::egui::{Pos2, Rect, Vec2, pos2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WindowId {
    pub handle: u64,
    pub process: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TargetWindow {
    pub id: WindowId,
    pub bounds: Rect,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Monitor {
    pub bounds: Rect,
    pub work_area: Rect,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Widget {
    pub outer: Rect,
    pub content: Rect,
    pub scale: f32,
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
    window: WindowId,
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

#[derive(Default)]
pub(crate) struct Placement {
    anchor: Option<Anchor>,
}

impl Placement {
    pub fn attached(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn detach(&mut self) {
        self.anchor = None;
    }

    pub fn snap(&mut self, widget: Widget, windows: &[TargetWindow]) -> Option<Pos2> {
        self.detach();
        // The first intersecting window is frontmost; never attach through it to another app.
        let target = windows.iter().find(|target| {
            target
                .bounds
                .expand(32.0 * widget.scale)
                .intersects(widget.content)
        })?;
        let anchor = [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right]
            .into_iter()
            .flat_map(|edge| {
                [false, true].map(|inside| Anchor {
                    window: target.id,
                    edge,
                    inside,
                    fraction: match edge {
                        Edge::Top | Edge::Bottom => fraction(
                            widget.content.left(),
                            target.bounds.left(),
                            target.bounds.width() - widget.content.width(),
                        ),
                        Edge::Left | Edge::Right => fraction(
                            widget.content.top(),
                            target.bounds.top(),
                            target.bounds.height() - widget.content.height(),
                        ),
                    },
                })
            })
            .map(|anchor| {
                let position =
                    anchor.position(target.bounds, widget.content.size(), 2.0 * widget.scale);
                (anchor, position.distance(widget.content.min))
            })
            .filter(|(_, distance)| *distance <= 32.0 * widget.scale)
            .min_by(|a, b| a.1.total_cmp(&b.1))?
            .0;
        self.anchor = Some(anchor);
        Some(at_anchor(widget, target.bounds, anchor))
    }

    pub fn update(&mut self, widget: Widget, windows: &[TargetWindow]) -> Option<Pos2> {
        let anchor = self.anchor?;
        let target = windows.iter().find(|target| target.id == anchor.window);
        match target {
            Some(target) => Some(at_anchor(widget, target.bounds, anchor)),
            None => {
                self.detach();
                None
            }
        }
    }
}

fn at_anchor(widget: Widget, target: Rect, anchor: Anchor) -> Pos2 {
    anchor.position(target, widget.content.size(), 2.0 * widget.scale)
        - (widget.content.min - widget.outer.min)
}

pub(crate) fn clamp_to_monitors(position: Pos2, size: Vec2, monitors: &[Monitor]) -> Pos2 {
    let center = position + size * 0.5;
    let Some(monitor) = monitors.iter().min_by(|a, b| {
        a.bounds
            .distance_sq_to_pos(center)
            .total_cmp(&b.bounds.distance_sq_to_pos(center))
    }) else {
        return position;
    };
    pos2(
        position.x.clamp(
            monitor.work_area.left(),
            (monitor.work_area.right() - size.x).max(monitor.work_area.left()),
        ),
        position.y.clamp(
            monitor.work_area.top(),
            (monitor.work_area.bottom() - size.y).max(monitor.work_area.top()),
        ),
    )
}

fn fraction(value: f32, start: f32, span: f32) -> f32 {
    if span > 0.0 {
        ((value - start) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
