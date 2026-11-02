use std::f32::consts::TAU;

use eframe::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, pos2, vec2};

use bongodsh_core::animation::HandPositions;

pub(crate) const INK: Color32 = Color32::from_rgb(42, 67, 82);
const BODY: Color32 = Color32::from_rgb(162, 215, 221);
const SHADE: Color32 = Color32::from_rgb(117, 182, 196);
const BELLY: Color32 = Color32::from_rgb(235, 248, 246);
const BLUSH: Color32 = Color32::from_rgb(238, 171, 160);

pub(crate) fn draw(painter: &Painter, bounds: Rect, hands: HandPositions) {
    let origin = bounds.min;
    let activity = hands.left.max(hands.right).clamp(0.0, 1.0);
    let body_center = origin + vec2(142.0, 124.0 + activity * 3.0);

    ellipse(
        painter,
        origin + vec2(233.0, 122.0),
        vec2(25.0, 12.0),
        -0.65,
        SHADE,
        3.0,
    );
    ellipse(
        painter,
        origin + vec2(240.0, 143.0),
        vec2(25.0, 12.0),
        0.55,
        BODY,
        3.0,
    );
    ellipse(
        painter,
        body_center,
        vec2(80.0 + activity * 2.0, 65.0 - activity * 3.0),
        0.0,
        BODY,
        3.0,
    );
    ellipse(
        painter,
        body_center + vec2(-4.0, 31.0),
        vec2(58.0, 29.0 - activity),
        0.0,
        BELLY,
        0.0,
    );
    ellipse(
        painter,
        body_center + vec2(-26.0, -35.0),
        vec2(23.0, 8.0),
        -0.35,
        Color32::from_rgb(197, 235, 237),
        0.0,
    );

    for x in [-25.0, 23.0] {
        let eye = body_center + vec2(x, -8.0);
        ellipse(painter, eye, vec2(4.5, 5.5 - activity * 1.8), 0.0, INK, 0.0);
        painter.circle_filled(eye + vec2(-1.2, -1.8), 1.3, Color32::WHITE);
        ellipse(
            painter,
            eye + vec2(if x < 0.0 { -11.0 } else { 11.0 }, 14.0),
            vec2(8.0, 4.0),
            0.0,
            BLUSH,
            0.0,
        );
    }
    let mouth = body_center + vec2(-1.0, 9.0);
    painter.add(Shape::CubicBezier(
        eframe::egui::epaint::CubicBezierShape::from_points_stroke(
            [
                mouth + vec2(-10.0, 0.0),
                mouth + vec2(-6.0, 9.0),
                mouth + vec2(6.0, 9.0),
                mouth + vec2(10.0, 0.0),
            ],
            false,
            Color32::TRANSPARENT,
            Stroke::new(2.4, INK),
        ),
    ));

    // Small water droplets give the original vector character its silhouette.
    ellipse(
        painter,
        origin + vec2(133.0, 47.0),
        vec2(4.0, 7.0),
        -0.45,
        BODY,
        2.0,
    );
    ellipse(
        painter,
        origin + vec2(149.0, 43.0),
        vec2(3.5, 5.5),
        0.4,
        BODY,
        2.0,
    );

    draw_hand(painter, origin + vec2(98.0, 184.0), hands.left, -0.12);
    draw_hand(painter, origin + vec2(181.0, 184.0), hands.right, 0.12);
}

fn draw_hand(painter: &Painter, center: Pos2, progress: f32, angle: f32) {
    let press = progress.clamp(-0.2, 1.15);
    let hand = center + vec2(0.0, press * 17.0);
    ellipse(
        painter,
        center + vec2(0.0, 25.0),
        vec2(21.0 + press * 3.0, 3.5),
        0.0,
        Color32::from_black_alpha(15),
        0.0,
    );
    ellipse(
        painter,
        hand,
        vec2(24.0 + press * 3.0, 19.0 - press * 5.0),
        angle * (1.0 - press),
        BELLY,
        3.0,
    );
    for x in [-7.0, 7.0] {
        painter.line_segment(
            [
                hand + vec2(x, 8.0 - press * 3.0),
                hand + vec2(x, 13.0 - press * 4.0),
            ],
            Stroke::new(1.6, SHADE),
        );
    }
}

fn ellipse(painter: &Painter, center: Pos2, radii: Vec2, angle: f32, fill: Color32, outline: f32) {
    let (sin, cos) = angle.sin_cos();
    let points = (0..48)
        .map(|index| {
            let t = TAU * index as f32 / 48.0;
            let local = vec2(t.cos() * radii.x, t.sin() * radii.y);
            center + vec2(local.x * cos - local.y * sin, local.x * sin + local.y * cos)
        })
        .collect();
    painter.add(Shape::convex_polygon(
        points,
        fill,
        Stroke::new(outline, INK),
    ));
}

pub(crate) fn hit_bounds(bounds: Rect) -> Rect {
    Rect::from_min_max(
        bounds.min + vec2(53.0, 35.0),
        bounds.min + vec2(269.0, 219.0),
    )
}

pub(crate) fn close_icon(painter: &Painter, bounds: Rect) {
    for sign in [-1.0, 1.0] {
        painter.line_segment(
            [
                bounds.center() + vec2(-4.0, sign * -4.0),
                bounds.center() + vec2(4.0, sign * 4.0),
            ],
            Stroke::new(1.5, INK),
        );
    }
}

pub(crate) fn hint_bounds(bounds: Rect) -> Rect {
    Rect::from_center_size(
        pos2(bounds.center().x, bounds.min.y + 241.0),
        vec2(214.0, 26.0),
    )
}
