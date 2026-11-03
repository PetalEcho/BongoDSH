use std::{f32::consts::FRAC_PI_2, time::Duration};

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2, vec2};

use crate::positioning::Highlight;

pub(super) fn show(ctx: &egui::Context, hint: Highlight, scale: f32) {
    let points = match hint {
        Highlight::Corner {
            corner,
            inward,
            lengths,
        } => corner_path(corner / scale, inward, lengths / scale),
        Highlight::Edge { start, end } => {
            let start = start / scale;
            let end = end / scale;
            let segments = (start.distance(end) / 3.0).ceil().max(1.0) as usize;
            (0..=segments)
                .map(|index| start.lerp(end, index as f32 / segments as f32))
                .collect()
        }
    };
    let bounds = Rect::from_points(&points).expand(8.0);
    let points: Vec<_> = points
        .into_iter()
        .map(|point| point - bounds.min.to_vec2())
        .collect();
    ctx.show_viewport_deferred(
        egui::ViewportId::from_hash_of("snap-corner-highlight"),
        egui::ViewportBuilder::default()
            .with_title("BongoDSH snap highlight")
            .with_position(bounds.min)
            .with_inner_size(bounds.size())
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_active(false)
            .with_taskbar(false)
            .with_mouse_passthrough(true),
        move |ui, class| {
            if class == egui::ViewportClass::EmbeddedWindow {
                return;
            }
            paint(ui, &points);
            // Only the visible light animates. The character and window observer keep their own cadence.
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        },
    );
}

fn corner_path(corner: Pos2, inward: Vec2, lengths: Vec2) -> Vec<Pos2> {
    let radius = 12.0_f32.min(lengths.x * 0.5).min(lengths.y * 0.5);
    let horizontal = lengths.x - radius;
    let arc = radius * FRAC_PI_2;
    let total = horizontal + arc + lengths.y - radius;
    let point = |distance: f32| {
        let offset = if distance < horizontal {
            vec2(lengths.x - distance, 0.0)
        } else if distance < horizontal + arc {
            let angle = (distance - horizontal) / radius;
            vec2(radius - radius * angle.sin(), radius - radius * angle.cos())
        } else {
            vec2(0.0, radius + distance - horizontal - arc)
        };
        corner + offset * inward
    };
    let segments = (total / 3.0).ceil().max(1.0) as usize;
    (0..=segments)
        .map(|index| point(index as f32 / segments as f32 * total))
        .collect()
}

fn paint(ui: &egui::Ui, points: &[Pos2]) {
    let time = ui.ctx().input(|input| input.time) as f32;
    let head = (time * 0.55).fract();
    let segments = points.len() - 1;
    for (index, segment) in points.windows(2).enumerate() {
        let t = index as f32 / segments as f32;
        let fade = (t * 10.0).min((1.0 - t) * 10.0).min(1.0);
        let trail = (head - t).rem_euclid(1.0);
        let light = (1.0 - trail / 0.28).max(0.0).powi(2);
        let alpha = ((70.0 + 185.0 * light) * fade) as u8;
        let color = Color32::from_rgba_unmultiplied(
            (100.0 + 135.0 * light) as u8,
            (175.0 + 70.0 * light) as u8,
            255,
            alpha,
        );
        for (width, strength) in [(9.0_f32, 0.08), (5.0, 0.18), (2.0, 1.0)] {
            ui.painter().line_segment(
                [segment[0], segment[1]],
                Stroke::new(width, color.gamma_multiply(strength)),
            );
        }
    }
}
