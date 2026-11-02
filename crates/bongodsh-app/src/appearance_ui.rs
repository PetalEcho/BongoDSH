use bongodsh_core::animation::HandPositions;
use eframe::egui::{
    self, Align2, Color32, FontId, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};

use crate::{
    appearance::{Appearance, Part, Problem},
    art,
};

pub(crate) const VIEWPORT: &str = "appearance";
const ACCENT: Color32 = Color32::from_rgb(46, 126, 136);
const MUTED: Color32 = Color32::from_rgb(103, 123, 134);
const LINE: Color32 = Color32::from_rgb(211, 226, 233);
const PAPER: Color32 = Color32::from_rgb(246, 250, 252);

#[derive(Default)]
pub(crate) struct AppearanceUi {
    pub(crate) open: bool,
    pub(crate) pick: Option<Part>,
    pub(crate) play: bool,
    target: Part,
    dark_preview: bool,
}

impl AppearanceUi {
    pub(crate) fn dropped(&mut self, ctx: &egui::Context, appearance: &mut Appearance) -> bool {
        let files = ctx.input(|input| input.raw.dropped_files.clone());
        if files.is_empty() {
            return false;
        }
        self.open = true;
        if files.len() != 1 {
            appearance.problem = Some(Problem::MultipleFiles);
        } else if let Some(path) = &files[0].path {
            // A drop on the pet itself always replaces the body, not a hidden hand selection.
            let part = if ctx.viewport_id() == egui::ViewportId::ROOT {
                Part::Body
            } else {
                self.target
            };
            appearance.import(path.clone(), part, ctx);
        } else {
            appearance.problem = Some(Problem::Read);
        }
        true
    }

    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        appearance: &mut Appearance,
        hands: HandPositions,
        chinese: bool,
    ) {
        if !self.open {
            return;
        }
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(VIEWPORT),
            egui::ViewportBuilder::default()
                .with_title(tr(
                    chinese,
                    "BongoDSH · 角色与图片",
                    "BongoDSH · Character & images",
                ))
                .with_inner_size([760.0, 610.0])
                .with_min_inner_size([420.0, 500.0]),
            |ui, _class| {
                if ui.ctx().input(|input| input.viewport().close_requested()) {
                    self.open = false;
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                    return;
                }
                self.dropped(ui.ctx(), appearance);
                ui.painter().rect_filled(ui.max_rect(), 0.0, PAPER);
                egui::Frame::central_panel(ui.style())
                    .fill(PAPER)
                    .inner_margin(22.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .show(ui, |ui| self.panel(ui, appearance, hands, chinese));
                    });
                if self.pick.is_some() || self.play {
                    // Native dialogs and demo playback are handled by the root viewport.
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
            },
        );
    }

    pub(crate) fn panel(
        &mut self,
        ui: &mut Ui,
        appearance: &mut Appearance,
        hands: HandPositions,
        chinese: bool,
    ) {
        ui.label(
            RichText::new(tr(chinese, "角色与图片", "Character & images"))
                .size(18.0)
                .strong(),
        );
        ui.label(
            RichText::new(tr(
                chinese,
                "选择内置角色，或组合自己的身体和双手。",
                "Choose a character or combine your own body and hand images.",
            ))
            .color(MUTED),
        );
        ui.add_space(18.0);
        if ui.available_width() >= 660.0 {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    vec2(302.0, 430.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.preview(ui, appearance, hands, chinese);
                    },
                );
                ui.add_space(18.0);
                ui.vertical(|ui| self.controls(ui, appearance, chinese));
            });
        } else {
            self.preview(ui, appearance, hands, chinese);
            ui.add_space(16.0);
            self.controls(ui, appearance, chinese);
        }
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(6.0);
        if appearance.busy() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(tr(
                    chinese,
                    "正在处理并保存图片…",
                    "Processing and saving images…",
                ));
            });
        } else if let Some(problem) = appearance.problem {
            ui.colored_label(Color32::from_rgb(161, 72, 56), problem.message(chinese));
        } else {
            ui.small(tr(
                chinese,
                "更改自动保存在本机，不上传图片。",
                "Changes are saved on this device. Images are never uploaded.",
            ));
        }
    }

    fn preview(
        &mut self,
        ui: &mut Ui,
        appearance: &Appearance,
        hands: HandPositions,
        chinese: bool,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr(chinese, "实时预览", "Live preview")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.toggle_value(&mut self.dark_preview, tr(chinese, "深色底", "Dark"));
            });
        });
        ui.add_space(8.0);
        let (preview, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), 258.0), Sense::hover());
        checkerboard(ui, preview, self.dark_preview);
        let canvas = Rect::from_min_size(
            pos2(preview.center().x - 150.0, preview.top()),
            vec2(300.0, 258.0),
        );
        art::draw_character(&ui.painter_at(preview), canvas, hands, appearance.layers());
        ui.painter()
            .rect_stroke(preview, 8.0, Stroke::new(1.0, LINE), StrokeKind::Inside);
        ui.add_space(10.0);
        let name = if appearance.uses_custom() {
            tr(chinese, "我的角色", "My character")
        } else {
            tr(chinese, "初始小鲸鱼", "Original whale")
        };
        ui.label(RichText::new(name).strong());
        if ui
            .add_sized(
                [ui.available_width(), 34.0],
                egui::Button::new(tr(chinese, "试播按键动作", "Preview typing animation")),
            )
            .clicked()
        {
            self.play = true;
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(tr(
                chinese,
                "手部随左右按键动作独立回弹。未添加手部时，仅身体轻动。",
                "Hand layers bounce independently. Without hand images, only the body moves.",
            ))
            .small()
            .color(MUTED),
        );
    }

    fn controls(&mut self, ui: &mut Ui, appearance: &mut Appearance, chinese: bool) {
        ui.add_enabled_ui(!appearance.busy(), |ui| {
            ui.label(RichText::new(tr(chinese, "内置角色", "Built-in characters")).strong());
            ui.add_space(6.0);
            if ui.selectable_label(!appearance.uses_custom(), tr(chinese, "小鲸鱼", "Whale")).clicked() {
                appearance.select_default(ui.ctx());
            }
            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(tr(chinese, "我的图片", "My images")).strong());
                if appearance.has_custom() && !appearance.uses_custom() && ui.small_button(tr(chinese, "使用此角色", "Use this character")).clicked() {
                    appearance.select_custom(ui.ctx());
                }
            });
            ui.small(tr(chinese, "身体必选，双手可选。不会自动补手。", "A body is required; hand images are optional."));
            ui.add_space(8.0);
            for part in Part::ALL {
                self.layer_row(ui, appearance, part, chinese);
                ui.add_space(5.0);
            }
            ui.add_space(4.0);
            let enabled = self.target == Part::Body || appearance.has_custom();
            let (drop_rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::click());
            let hovered_file = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
            ui.painter().rect_filled(drop_rect, 8.0, if hovered_file { Color32::from_rgb(216, 238, 239) } else { Color32::from_rgb(232, 241, 245) });
            ui.painter().rect_stroke(drop_rect, 8.0, Stroke::new(1.0, if hovered_file { ACCENT } else { LINE }), StrokeKind::Inside);
            let text = if chinese { format!("拖入图片或点击选择 · {}", self.target.label(true)) } else { format!("Drop or choose an image · {}", self.target.label(false)) };
            ui.painter().text(drop_rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(13.0), if enabled { art::INK } else { MUTED });
            if response.clicked() && enabled { self.pick = Some(self.target); }
            ui.add_space(7.0);
            ui.label(RichText::new(tr(chinese, "PNG / JPEG / WebP，最大 16 MB，最长边 4096 px。透明图片效果更好。", "PNG / JPEG / WebP, up to 16 MB and 4096 px per side. Transparent images work best.")).small().color(MUTED));
        });
    }

    fn layer_row(&mut self, ui: &mut Ui, appearance: &mut Appearance, part: Part, chinese: bool) {
        let enabled = part == Part::Body || appearance.has_custom();
        let selected = self.target == part;
        egui::Frame::new()
            .fill(if selected {
                Color32::from_rgb(229, 241, 243)
            } else {
                Color32::WHITE
            })
            .stroke(Stroke::new(1.0, if selected { ACCENT } else { LINE }))
            .corner_radius(7.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        let (thumb, response) =
                            ui.allocate_exact_size(vec2(42.0, 42.0), Sense::click());
                        if let Some(texture) = appearance.custom_layer(part) {
                            art::paint_texture(
                                ui.painter(),
                                texture,
                                art::fit_image(thumb.shrink(2.0), texture.size_vec2()),
                            );
                        } else {
                            ui.painter().text(
                                thumb.center(),
                                Align2::CENTER_CENTER,
                                "+",
                                FontId::proportional(22.0),
                                MUTED,
                            );
                        }
                        if response.clicked() {
                            self.target = part;
                            self.pick = Some(part);
                        }
                        ui.vertical(|ui| {
                            if ui.selectable_label(selected, part.label(chinese)).clicked() {
                                self.target = part;
                            }
                            if let Some(texture) = appearance.custom_layer(part) {
                                ui.small(format!(
                                    "{} × {} px",
                                    texture.size()[0],
                                    texture.size()[1]
                                ));
                            } else {
                                ui.small(if part == Part::Body {
                                    tr(chinese, "待导入", "Required")
                                } else {
                                    tr(chinese, "未添加", "Optional")
                                });
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if part != Part::Body
                                && appearance.custom_layer(part).is_some()
                                && ui.small_button(tr(chinese, "移除", "Remove")).clicked()
                            {
                                appearance.remove(part, ui.ctx());
                            }
                            if ui
                                .small_button(if appearance.custom_layer(part).is_some() {
                                    tr(chinese, "替换", "Replace")
                                } else {
                                    tr(chinese, "选择", "Choose")
                                })
                                .clicked()
                            {
                                self.target = part;
                                self.pick = Some(part);
                            }
                        });
                    });
                });
            });
    }
}

fn checkerboard(ui: &Ui, bounds: Rect, dark: bool) {
    let colors = if dark {
        [Color32::from_rgb(52, 68, 79), Color32::from_rgb(61, 78, 89)]
    } else {
        [
            Color32::from_rgb(235, 241, 245),
            Color32::from_rgb(248, 251, 253),
        ]
    };
    let painter = ui.painter_at(bounds);
    for row in 0..(bounds.height() / 16.0).ceil() as i32 {
        for column in 0..(bounds.width() / 16.0).ceil() as i32 {
            painter.rect_filled(
                Rect::from_min_size(
                    bounds.min + vec2(column as f32 * 16.0, row as f32 * 16.0),
                    vec2(16.0, 16.0),
                ),
                0.0,
                colors[((row + column) % 2) as usize],
            );
        }
    }
}

fn tr(chinese: bool, zh: &'static str, en: &'static str) -> &'static str {
    if chinese { zh } else { en }
}
