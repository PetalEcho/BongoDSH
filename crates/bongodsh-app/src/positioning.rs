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
    pub maximized: bool,
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
pub(crate) enum Highlight {
    Corner {
        corner: Pos2,
        inward: Vec2,
        lengths: Vec2,
    },
    Edge {
        start: Pos2,
        end: Pos2,
    },
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
    pub fn target(&self) -> Option<WindowId> {
        self.anchor.map(|anchor| anchor.window)
    }

    pub fn attached(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn detach(&mut self) {
        self.anchor = None;
    }

    pub fn highlight(
        &self,
        widget: Widget,
        windows: &[TargetWindow],
        monitors: &[Monitor],
    ) -> Option<Highlight> {
        let anchor = self.anchor?;
        let target = windows.iter().find(|target| target.id == anchor.window)?;
        // Keep maximized/fullscreen windows in the occlusion list, but never decorate them.
        // macOS exposes other apps' geometry without requiring Accessibility permission.
        if anchor.inside
            || target.maximized
            || monitors.iter().any(|monitor| {
                [monitor.bounds, monitor.work_area]
                    .into_iter()
                    .any(|area| target.bounds.contains_rect(area.shrink(2.0 * widget.scale)))
            })
            || widget.outer.intersect(target.bounds).is_positive()
        {
            return None;
        }
        let target = target.bounds;
        let (gap, start, end, near, far) = match anchor.edge {
            Edge::Top => (
                target.top() - widget.outer.bottom(),
                target.left_top(),
                target.right_top(),
                widget.outer.left(),
                widget.outer.right(),
            ),
            Edge::Bottom => (
                widget.outer.top() - target.bottom(),
                target.left_bottom(),
                target.right_bottom(),
                widget.outer.left(),
                widget.outer.right(),
            ),
            Edge::Left => (
                target.left() - widget.outer.right(),
                target.left_top(),
                target.left_bottom(),
                widget.outer.top(),
                widget.outer.bottom(),
            ),
            Edge::Right => (
                widget.outer.left() - target.right(),
                target.right_top(),
                target.right_bottom(),
                widget.outer.top(),
                widget.outer.bottom(),
            ),
        };
        let horizontal = matches!(anchor.edge, Edge::Top | Edge::Bottom);
        let edge_start = if horizontal { start.x } else { start.y };
        let edge_end = if horizontal { end.x } else { end.y };
        let threshold = 32.0 * widget.scale;
        if !(0.0..=threshold).contains(&gap)
            || far < edge_start - threshold
            || near > edge_end + threshold
        {
            return None;
        }
        let start_distance = (edge_start - far).max(near - edge_start).max(0.0);
        let end_distance = (edge_end - far).max(near - edge_end).max(0.0);
        if start_distance.min(end_distance) > threshold {
            let center = ((near + far) * 0.5).clamp(edge_start, edge_end);
            let half_length = ((far - near) * 0.5).clamp(40.0 * widget.scale, 90.0 * widget.scale);
            let from = (center - half_length).max(edge_start + 12.0 * widget.scale);
            let to = (center + half_length).min(edge_end - 12.0 * widget.scale);
            return Some(Highlight::Edge {
                start: if horizontal {
                    pos2(from, start.y)
                } else {
                    pos2(start.x, from)
                },
                end: if horizontal {
                    pos2(to, start.y)
                } else {
                    pos2(start.x, to)
                },
            });
        }
        let corner = if start_distance <= end_distance {
            start
        } else {
            end
        };
        let right = corner.x == target.right();
        let bottom = corner.y == target.bottom();
        Some(Highlight::Corner {
            corner,
            inward: Vec2::new(
                if right { -1.0 } else { 1.0 },
                if bottom { -1.0 } else { 1.0 },
            ),
            lengths: Vec2::new(
                (target.width() * 0.5).min(180.0 * widget.scale),
                (target.height() * 0.5).min(110.0 * widget.scale),
            ),
        })
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

/// Match public accessibility geometry to one native window, never just its app.
#[cfg(target_os = "macos")]
pub(crate) fn identify_focused_window(
    process: u32,
    bounds: Rect,
    windows: &[TargetWindow],
) -> Option<WindowId> {
    let mut matching = windows.iter().filter(|window| {
        window.id.process == process
            && (window.bounds.min - bounds.min).abs().max_elem() <= 1.0
            && (window.bounds.max - bounds.max).abs().max_elem() <= 1.0
    });
    let window = matching.next()?;
    matching.next().is_none().then_some(window.id)
}
