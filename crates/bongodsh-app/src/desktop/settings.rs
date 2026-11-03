use eframe::egui::{
    self, Align, Color32, FontId, Layout, RichText, Stroke, ViewportCommand, WindowLevel, vec2,
};

use super::{
    Desktop,
    controls::{checkbox_row, size_slider},
};
use crate::sprites::Artwork;

const LABELS: [(&str, &str); 6] = [
    ("基础女仆鱼", "Fish maid"),
    ("水手装", "Sailor"),
    ("睡衣装", "Pajamas"),
    ("魔法师装", "Wizard"),
    ("长发蓝女仆", "Blue maid"),
    ("克劳德", "Claude"),
];

struct Palette {
    background: Color32,
    text: Color32,
    muted: Color32,
    border: Color32,
    control: Color32,
    accent: Color32,
    error: Color32,
}

impl Palette {
    fn new(dark: bool) -> Self {
        if dark {
            Self {
                background: Color32::from_rgb(0x24, 0x3b, 0x46),
                text: Color32::from_rgb(0xe4, 0xf3, 0xf6),
                muted: Color32::from_rgb(0xb6, 0xcc, 0xd5),
                border: Color32::from_rgb(0x53, 0x75, 0x81),
                control: Color32::from_rgb(0x34, 0x53, 0x60),
                accent: Color32::from_rgb(0x74, 0xbd, 0xd0),
                error: Color32::from_rgb(0xf5, 0xbe, 0xa0),
            }
        } else {
            Self {
                background: Color32::from_rgb(0xf5, 0xfb, 0xfc),
                text: Color32::from_rgb(0x2a, 0x43, 0x52),
                muted: Color32::from_rgb(0x53, 0x6b, 0x77),
                border: Color32::from_rgb(0xb7, 0xd4, 0xdb),
                control: Color32::from_rgb(0xe3, 0xf1, 0xf4),
                accent: Color32::from_rgb(0x27, 0x76, 0x8b),
                error: Color32::from_rgb(0x9b, 0x49, 0x2b),
            }
        }
    }

    fn apply(&self, ui: &mut egui::Ui) {
        let style = ui.style_mut();
        style.override_font_id = Some(FontId::proportional(13.0));
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(8.0, 6.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.icon_width = 16.0;
        style.visuals.override_text_color = Some(self.text);
        style.visuals.window_fill = self.background;
        style.visuals.panel_fill = self.background;
        style.visuals.extreme_bg_color = self.control;
        style.visuals.selection.bg_fill = self.accent;
        style.visuals.selection.stroke = Stroke::new(1.0, self.text);
        for widget in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.bg_fill = self.control;
            widget.weak_bg_fill = self.control;
            widget.bg_stroke = Stroke::new(1.0, self.border);
            widget.fg_stroke = Stroke::new(1.0, self.text);
            widget.corner_radius = 7.into();
        }
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.5, self.accent);
    }

    fn note(&self, ui: &mut egui::Ui, message: &str) {
        ui.label(RichText::new(message).size(11.0).color(self.muted));
    }
}

impl Desktop {
    pub(super) fn desktop_window(&mut self, ctx: &egui::Context) {
        if !self.desktop_settings_open {
            return;
        }
        let max_size = self
            .docking
            .panel_geometry(vec2(264.0, 10000.0))
            .map_or(vec2(264.0, 600.0), |(_, size)| size);
        let size = self.desktop_settings_size.min(max_size);
        let mut builder = egui::ViewportBuilder::default()
            .with_title(self.text("BongoDSH 设置", "BongoDSH settings"))
            .with_inner_size(size)
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_active(true);
        if let Some((position, _)) = self.docking.panel_geometry(size) {
            builder = builder.with_position(position);
        }
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("desktop-settings"),
            builder,
            |ui, _class| {
                let (focused, close) = ui.ctx().input_mut(|input| {
                    (
                        input.focused,
                        input.viewport().close_requested()
                            || input.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                    )
                });
                if close || (self.desktop_settings_focused && !focused) {
                    self.close_desktop_settings(ui.ctx());
                    return;
                }
                self.desktop_settings_focused |= focused;
                let palette = Palette::new(ui.visuals().dark_mode);
                palette.apply(ui);
                // Measure the content before choosing the viewport height; expansion remains scrollable on a short display.
                ui.set_max_size(max_size);
                let response = egui::Frame::new()
                    .fill(palette.background)
                    .stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(12)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width((max_size.x - 30.0).max(1.0));
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .max_height((max_size.y - 30.0).max(1.0))
                            .show(ui, |ui| self.desktop_panel(ui, &palette));
                    });
                let fitted = vec2(max_size.x, response.response.rect.height().ceil()).min(max_size);
                if (fitted - self.desktop_settings_size).length() > 0.5 {
                    self.desktop_settings_size = fitted;
                    ui.ctx()
                        .send_viewport_cmd(ViewportCommand::InnerSize(fitted));
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
            },
        );
    }

    fn close_desktop_settings(&mut self, ctx: &egui::Context) {
        self.desktop_settings_open = false;
        self.desktop_settings_focused = false;
        self.reset();
        ctx.request_repaint_of(egui::ViewportId::ROOT);
    }

    fn desktop_panel(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("BongoDSH").size(15.0).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(RichText::new("×").size(19.0)).frame(false))
                    .on_hover_text(self.text("关闭设置", "Close settings"))
                    .clicked()
                {
                    self.close_desktop_settings(ui.ctx());
                }
            });
        });
        ui.label(
            RichText::new(self.text(
                "打字时，角色随你的节奏轻轻动一动。",
                "Your companion moves gently as you type.",
            ))
            .size(12.0)
            .color(palette.muted),
        );
        ui.add_space(4.0);
        ui.label(self.text("角色", "Character"));
        let mut character = self.character;
        let label = LABELS[character];
        egui::ComboBox::from_id_salt("desktop-character")
            .width(ui.available_width())
            .selected_text(self.text(label.0, label.1))
            .show_ui(ui, |ui| {
                for (index, (chinese, english)) in LABELS.into_iter().enumerate() {
                    ui.selectable_value(&mut character, index, self.text(chinese, english));
                }
            });
        if character != self.character {
            match Artwork::load(ui.ctx(), character) {
                Ok(artwork) => {
                    self.artwork = Some(artwork);
                    self.character = character;
                    self.character_error = None;
                    self.demo = None;
                    self.restart_input(ui.ctx());
                    self.docking.invalidate_geometry();
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
                Err(error) => self.character_error = Some(error),
            }
        }
        palette.note(
            ui,
            self.text("身体与双手独立动作。", "Body and hands move independently."),
        );
        if let Some(error) = &self.character_error {
            ui.colored_label(palette.error, error);
        }
        let mut paused = self.paused;
        if checkbox_row(ui, self.text("暂停动作", "Pause animation"), &mut paused) {
            self.paused = paused;
            self.demo = None;
            self.restart_input(ui.ctx());
            ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
        }
        let mut docked = self.docking.enabled();
        ui.add_enabled_ui(self.docking.available(), |ui| {
            if checkbox_row(ui, self.text("吸附窗口", "Snap to windows"), &mut docked) {
                self.docking.set_enabled(docked);
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
        });
        ui.horizontal(|ui| {
            ui.label(self.text("角色大小", "Character size"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} px", self.character_size as u16))
                        .color(palette.muted),
                );
            });
        });
        ui.spacing_mut().slider_width = ui.available_width();
        if size_slider(ui, &mut self.character_size).changed() {
            self.character_size = self.character_size.round();
            ui.ctx().send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                ViewportCommand::InnerSize(vec2(
                    self.character_size,
                    self.character_size * 252.0 / 300.0,
                )),
            );
            self.docking.invalidate_geometry();
            ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            if ui
                .add_sized(
                    [width, 30.0],
                    egui::Button::new(self.text("试一下", "Try it")),
                )
                .clicked()
            {
                self.start_demo();
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
            if ui
                .add_sized(
                    [width, 30.0],
                    egui::Button::new(self.text("恢复位置", "Reset position")),
                )
                .clicked()
            {
                self.docking.restore_initial();
                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
            }
        });
        if ui
            .add_sized(
                [ui.available_width(), 30.0],
                egui::Button::new(self.text("返回 DSH", "Return to DSH")),
            )
            .clicked()
        {
            ui.ctx()
                .send_viewport_cmd_to(egui::ViewportId::ROOT, ViewportCommand::Close);
        }
        palette.note(ui, self.docking.status(self.chinese_font));
        ui.add_space(4.0);
        egui::CollapsingHeader::new(self.text("桌面选项", "Desktop options"))
            .default_open(false)
            .show(ui, |ui| {
                let top_label = self.text("保持置顶", "Always on top");
                if checkbox_row(ui, top_label, &mut self.always_on_top) {
                    ui.ctx().send_viewport_cmd_to(
                        egui::ViewportId::ROOT,
                        ViewportCommand::WindowLevel(if self.always_on_top {
                            WindowLevel::AlwaysOnTop
                        } else {
                            WindowLevel::Normal
                        }),
                    );
                }
                self.keyboard_panel(ui);
            });
    }
}
