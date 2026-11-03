use bongodsh_core::animation::HandPositions;
use eframe::egui::{
    self, Color32, ColorImage, Painter, Rect, TextureHandle, TextureOptions, Vec2, pos2, vec2,
};

const CHARACTERS: [[&[u8]; 3]; 6] = [
    [
        include_bytes!("../../../assets/concepts/fish-maid-colored.png"),
        include_bytes!("../../../assets/concepts/fish-maid-left-hand-colored.png"),
        include_bytes!("../../../assets/concepts/fish-maid-right-hand-colored.png"),
    ],
    [
        include_bytes!("../../../assets/characters/sailor/body-colored.png"),
        include_bytes!("../../../assets/characters/sailor/left-hand-colored.png"),
        include_bytes!("../../../assets/characters/sailor/right-hand-colored.png"),
    ],
    [
        include_bytes!("../../../assets/characters/pajamas/body-colored.png"),
        include_bytes!("../../../assets/characters/pajamas/left-hand-colored.png"),
        include_bytes!("../../../assets/characters/pajamas/right-hand-colored.png"),
    ],
    [
        include_bytes!("../../../assets/characters/wizard/body-colored.png"),
        include_bytes!("../../../assets/characters/wizard/left-hand-colored.png"),
        include_bytes!("../../../assets/characters/wizard/right-hand-colored.png"),
    ],
    [
        include_bytes!("../../../assets/concepts/blue-maid-prone-colored.png"),
        include_bytes!("../../../assets/concepts/blue-maid-prone-left-hand-colored.png"),
        include_bytes!("../../../assets/concepts/blue-maid-prone-right-hand-colored.png"),
    ],
    [
        include_bytes!("../../../assets/concepts/orange-bride-prone-colored.png"),
        include_bytes!("../../../assets/concepts/orange-bride-prone-left-hand-colored.png"),
        include_bytes!("../../../assets/concepts/orange-bride-prone-right-hand-colored.png"),
    ],
];

// Keep the same visible-layer geometry as plugins/dsh-bongodsh/src/characters.js.
const HANDS: [([f32; 2], Vec2); 6] = [
    ([56.0, 188.0], vec2(36.0, 28.0)),
    ([74.0, 170.0], vec2(38.0, 29.0)),
    ([67.0, 173.0], vec2(36.0, 28.0)),
    ([73.0, 169.0], vec2(37.0, 29.0)),
    ([45.0, 161.0], vec2(34.0, 27.0)),
    ([65.0, 147.0], vec2(36.0, 28.0)),
];

struct Sprite {
    texture: TextureHandle,
    uv: Rect,
    size: Vec2,
}

impl Sprite {
    fn load(ctx: &egui::Context, bytes: &[u8]) -> Result<Self, String> {
        let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .map_err(|error| format!("角色图片无法解码：{error}"))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel[3] >= 16 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
        if right <= left || bottom <= top {
            return Err("角色图片没有可见内容。".into());
        }
        let uv = Rect::from_min_max(
            pos2(left as f32 / width as f32, top as f32 / height as f32),
            pos2(right as f32 / width as f32, bottom as f32 / height as f32),
        );
        let color =
            ColorImage::from_rgba_unmultiplied([width as usize, height as usize], image.as_raw());
        Ok(Self {
            texture: ctx.load_texture("character-layer", color, TextureOptions::LINEAR),
            uv,
            size: vec2((right - left) as f32, (bottom - top) as f32),
        })
    }

    fn fit(&self, bounds: Rect) -> Rect {
        let scale = (bounds.width() / self.size.x).min(bounds.height() / self.size.y);
        let size = self.size * scale;
        Rect::from_min_size(
            pos2(bounds.center().x - size.x * 0.5, bounds.bottom() - size.y),
            size,
        )
    }
}

pub(crate) struct Artwork {
    layers: [Sprite; 3],
    centers: [f32; 2],
    hand_size: Vec2,
}

impl Artwork {
    pub fn load(ctx: &egui::Context, character: usize) -> Result<Self, String> {
        let sources = CHARACTERS.get(character).ok_or("未知角色。")?;
        let (centers, hand_size) = HANDS[character];
        Ok(Self {
            layers: [
                Sprite::load(ctx, sources[0])?,
                Sprite::load(ctx, sources[1])?,
                Sprite::load(ctx, sources[2])?,
            ],
            centers,
            hand_size,
        })
    }

    fn rects(&self) -> [Rect; 3] {
        [
            self.layers[0].fit(Rect::from_min_size(pos2(6.0, 6.0), vec2(288.0, 242.0))),
            self.layers[1].fit(Rect::from_min_size(
                pos2(
                    self.centers[0] - self.hand_size.x / 2.0,
                    248.0 - self.hand_size.y,
                ),
                self.hand_size,
            )),
            self.layers[2].fit(Rect::from_min_size(
                pos2(
                    self.centers[1] - self.hand_size.x / 2.0,
                    248.0 - self.hand_size.y,
                ),
                self.hand_size,
            )),
        ]
    }

    pub fn content_bounds(&self, bounds: Rect) -> Rect {
        let [body, left, right] = self.rects();
        screen_rect(body.union(left).union(right), bounds)
    }

    pub fn animated_rects(
        &self,
        positions: HandPositions,
        hand_amplitude: u16,
        body_amplitude: u16,
    ) -> [Rect; 3] {
        let mut rects = self.rects();
        let hand_factor = f32::from(hand_amplitude.min(200)) / 100.0;
        let body_factor = f32::from(body_amplitude.min(200)) / 100.0;
        let activity = positions.left.max(positions.right).clamp(0.0, 1.0) * body_factor;
        rects[0] = transform(
            rects[0],
            pos2(150.0, 248.0),
            vec2(1.0 + activity * 0.02, 1.0 - activity * 0.035),
            Vec2::ZERO,
        );
        for (index, press) in [positions.left, positions.right].into_iter().enumerate() {
            let press = press.clamp(-0.2, 1.15) * hand_factor;
            rects[index + 1] = transform(
                rects[index + 1],
                pos2(self.centers[index], 248.0),
                vec2(1.0 + press * 0.06, 1.0 - press * 0.15),
                vec2(0.0, (press * 0.7).min(1.2)),
            );
        }
        rects
    }

    pub fn draw(
        &self,
        painter: &Painter,
        bounds: Rect,
        positions: HandPositions,
        hand_amplitude: u16,
        body_amplitude: u16,
    ) {
        for (sprite, rect) in
            self.layers
                .iter()
                .zip(self.animated_rects(positions, hand_amplitude, body_amplitude))
        {
            painter.image(
                sprite.texture.id(),
                screen_rect(rect, bounds),
                sprite.uv,
                Color32::WHITE,
            );
        }
    }
}

fn transform(rect: Rect, origin: egui::Pos2, scale: Vec2, offset: Vec2) -> Rect {
    Rect::from_min_max(
        origin + (rect.min - origin) * scale + offset,
        origin + (rect.max - origin) * scale + offset,
    )
}

fn screen_rect(rect: Rect, bounds: Rect) -> Rect {
    let scale = bounds.width() / 300.0;
    Rect::from_min_max(
        bounds.min + rect.min.to_vec2() * scale,
        bounds.min + rect.max.to_vec2() * scale,
    )
}
