use std::f32::consts::TAU;

use eframe::egui::{
    Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2, emath::TSTransform, pos2, vec2,
};

use bongodsh_core::{Hand, animation::HandPositions};

pub(crate) const INK: Color32 = Color32::from_rgb(42, 67, 82);
const BODY: Color32 = Color32::from_rgb(162, 215, 221);
const SHADE: Color32 = Color32::from_rgb(117, 182, 196);
const BELLY: Color32 = Color32::from_rgb(235, 248, 246);
const BLUSH: Color32 = Color32::from_rgb(238, 171, 160);

#[derive(Clone, Copy)]
pub(crate) struct CharacterLayout {
    transform: TSTransform,
}

impl CharacterLayout {
    pub(crate) fn new(bounds: Rect, scale: f32) -> Self {
        Self {
            transform: TSTransform::new(
                vec2(bounds.center().x - 150.0 * scale, bounds.min.y),
                scale,
            ),
        }
    }

    pub(crate) fn hit_bounds(self) -> Rect {
        self.transform * Rect::from_min_max(pos2(53.0, 35.0), pos2(269.0, 234.0))
    }

    pub(crate) fn hand_at(self, point: Pos2, hands: HandPositions, travel: f32) -> Option<Hand> {
        let local = self.transform.inverse() * point;
        for (hand, x, progress) in [
            (Hand::Left, 98.0, hands.left),
            (Hand::Right, 181.0, hands.right),
        ] {
            let (center, _, _) = hand_pose(pos2(x, 184.0), progress, hand, travel);
            if ((local - center) / vec2(30.0, 23.0)).length_sq() <= 1.0 {
                return Some(hand);
            }
        }
        None
    }
}

struct Drawing<'a> {
    painter: &'a Painter,
    transform: TSTransform,
}

impl Drawing<'_> {
    fn add(&self, mut shape: Shape) {
        shape.transform(self.transform);
        self.painter.add(shape);
    }

    fn circle_filled(&self, center: Pos2, radius: f32, color: Color32) {
        self.add(Shape::circle_filled(center, radius, color));
    }

    fn line_segment(&self, points: [Pos2; 2], stroke: Stroke) {
        self.add(Shape::line_segment(points, stroke));
    }
}

pub(crate) fn draw(painter: &Painter, layout: CharacterLayout, hands: HandPositions, travel: f32) {
    let drawing = Drawing {
        painter,
        transform: layout.transform,
    };
    let painter = &drawing;
    let origin = Pos2::ZERO;
    let activity = ((hands.left + hands.right) * 0.5).clamp(-0.16, 1.0);
    let body_center = origin + vec2(142.0, 124.0 + activity * 1.3);

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
        vec2(80.0 + activity * 0.6, 65.0 - activity * 0.8),
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
        ellipse(painter, eye, vec2(4.5, 5.5 - activity * 0.4), 0.0, INK, 0.0);
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

    draw_hand(
        painter,
        origin + vec2(98.0, 184.0),
        hands.left,
        Hand::Left,
        travel,
    );
    draw_hand(
        painter,
        origin + vec2(181.0, 184.0),
        hands.right,
        Hand::Right,
        travel,
    );
}

fn hand_pose(center: Pos2, progress: f32, side: Hand, travel: f32) -> (Pos2, Vec2, f32) {
    let press = progress.clamp(-0.16, 1.12);
    let direction = if side == Hand::Left { -1.0 } else { 1.0 };
    let contact = ((press - 0.55) / 0.45).clamp(0.0, 1.2);
    let hand = center + vec2(direction * press * 2.5, press * 20.0 * travel);
    let radii = vec2(23.0 + contact * 3.0, 17.0 - contact * 4.5);
    (hand, radii, direction * (0.16 - press * 0.23))
}

fn draw_hand(painter: &Drawing<'_>, center: Pos2, progress: f32, side: Hand, travel: f32) {
    let press = progress.clamp(-0.16, 1.12);
    let (hand, radii, angle) = hand_pose(center, progress, side, travel);
    let contact = press.clamp(0.0, 1.0);
    ellipse(
        painter,
        center + vec2(0.0, 20.0 * travel + 14.0),
        vec2(15.0 + contact * 8.0, 2.0 + contact * 1.5),
        0.0,
        Color32::from_black_alpha((7.0 + contact * 14.0) as u8),
        0.0,
    );
    ellipse(painter, hand, radii, angle, BELLY, 3.0);
    let rotation = eframe::egui::emath::Rot2::from_angle(angle);
    for x in [-7.0, 7.0] {
        painter.line_segment(
            [
                hand + rotation * vec2(x, radii.y * 0.35),
                hand + rotation * vec2(x, radii.y * 0.64),
            ],
            Stroke::new(1.6, SHADE),
        );
    }
}

fn ellipse(
    painter: &Drawing<'_>,
    center: Pos2,
    radii: Vec2,
    angle: f32,
    fill: Color32,
    outline: f32,
) {
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

pub(crate) fn menu_icon(painter: &Painter, bounds: Rect) {
    for y in [-4.0, 0.0, 4.0] {
        painter.line_segment(
            [
                bounds.center() + vec2(-5.0, y),
                bounds.center() + vec2(5.0, y),
            ],
            Stroke::new(1.5, INK),
        );
    }
}
