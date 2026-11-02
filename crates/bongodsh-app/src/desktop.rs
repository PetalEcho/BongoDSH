use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, IconData, Rect, Sense,
    ViewportCommand, WindowLevel, pos2, vec2,
};

use bongodsh_core::{InputEvent, KeyId, TypingState, animation::HandAnimation};

use crate::{art, docking::Docking, input};

pub(crate) fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BongoDSH")
            .with_inner_size([300.0, 270.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_always_on_top()
            .with_icon(IconData::default()),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "BongoDSH",
        options,
        Box::new(|creation| Ok(Box::new(Desktop::new(creation)))),
    )
}

struct Demo {
    remaining: u8,
    next_at: Instant,
}

struct Desktop {
    typing: TypingState,
    animation: HandAnimation,
    last_frame: Instant,
    demo: Option<Demo>,
    paused: bool,
    focused: bool,
    always_on_top: bool,
    chinese_font: bool,
    docking: Docking,
}

impl Desktop {
    fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let ctx = &creation.egui_ctx;
        ctx.set_visuals(egui::Visuals::light());
        ctx.global_style_mut(|style| {
            style.visuals.window_fill = Color32::from_rgb(243, 250, 251);
            style.visuals.override_text_color = Some(art::INK);
            style.visuals.selection.bg_fill = Color32::from_rgb(162, 215, 221);
            style.spacing.button_padding = vec2(10.0, 7.0);
        });
        Self {
            typing: TypingState::default(),
            animation: HandAnimation::default(),
            last_frame: Instant::now(),
            demo: None,
            paused: false,
            focused: false,
            always_on_top: true,
            chinese_font: install_system_font(ctx),
            docking: Docking::new(creation),
        }
    }

    fn text(&self, chinese: &'static str, english: &'static str) -> &'static str {
        if self.chinese_font { chinese } else { english }
    }

    fn apply(&mut self, event: InputEvent) {
        self.animation.apply(self.typing.apply(event));
    }

    fn reset(&mut self) {
        self.typing.apply(InputEvent::Reset);
        self.animation = HandAnimation::default();
    }

    fn start_demo(&mut self) {
        self.reset();
        self.paused = false;
        self.demo = Some(Demo {
            remaining: 8,
            next_at: Instant::now(),
        });
    }

    fn advance_demo(&mut self, now: Instant, ctx: &egui::Context) {
        let Some(demo) = &mut self.demo else { return };
        if now < demo.next_at {
            ctx.request_repaint_after(demo.next_at - now);
            return;
        }
        let key = KeyId::new(u32::from(demo.remaining));
        demo.remaining -= 1;
        demo.next_at = now + Duration::from_millis(190);
        let finished = demo.remaining == 0;
        self.apply(InputEvent::Press { key, repeat: false });
        self.apply(InputEvent::Release(key));
        if finished {
            self.demo = None;
        } else {
            ctx.request_repaint_after(Duration::from_millis(190));
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(185.0);
        ui.label(egui::RichText::new("BongoDSH").strong());
        ui.label(self.text("仅响应本窗口按键", "Keyboard input in this window"));
        ui.separator();
        if ui
            .button(self.text("演示双手动作", "Play hand animation"))
            .clicked()
        {
            self.start_demo();
            ui.ctx().request_repaint();
            ui.close();
        }
        let pause_label = if self.paused {
            self.text("继续动画", "Resume")
        } else {
            self.text("暂停动画", "Pause")
        };
        if ui.button(pause_label).clicked() {
            self.paused = !self.paused;
            self.demo = None;
            self.reset();
            ui.close();
        }
        let top_label = self.text("保持置顶", "Always on top");
        if ui.checkbox(&mut self.always_on_top, top_label).changed() {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::WindowLevel(if self.always_on_top {
                    WindowLevel::AlwaysOnTop
                } else {
                    WindowLevel::Normal
                }));
        }
        ui.menu_button(self.text("吸附与位置", "Snapping and position"), |ui| {
            ui.set_max_width(255.0);
            egui::ScrollArea::vertical()
                .max_height(225.0)
                .show(ui, |ui| {
                    self.docking.menu(ui, self.chinese_font);
                });
        });
        ui.separator();
        if ui.button(self.text("退出挂件", "Quit BongoDSH")).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

impl eframe::App for Desktop {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        ctx.input(|state| {
            if !state.focused && self.focused {
                self.reset();
            }
            self.focused = state.focused;
            if state.focused && !self.paused {
                for event in &state.events {
                    if let Some(input) = input::normalize(event) {
                        if self.demo.take().is_some() {
                            self.reset();
                        }
                        self.apply(input);
                    }
                }
            }
        });
        self.advance_demo(now, ctx);
        if self
            .animation
            .advance(now.saturating_duration_since(self.last_frame))
        {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        self.last_frame = now;
        self.docking.update(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bounds = ui.max_rect();
        let response = ui.interact(
            art::hit_bounds(bounds),
            ui.id().with("character"),
            // Typing Enter/Space must animate hands, not activate a focused button.
            Sense::CLICK | Sense::DRAG,
        );
        if response.drag_started_by(egui::PointerButton::Primary) {
            self.docking.begin_drag();
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            ui.ctx().request_repaint();
        }
        response.context_menu(|ui| {
            egui::ScrollArea::vertical()
                .max_height(245.0)
                .show(ui, |ui| self.menu(ui));
        });
        let response = response.on_hover_cursor(egui::CursorIcon::Grab);
        art::draw(ui.painter(), bounds, self.animation.positions());

        if response.hovered() || self.focused || self.paused || self.demo.is_some() {
            let close_rect = Rect::from_min_size(bounds.min + vec2(260.0, 12.0), vec2(25.0, 25.0));
            let close = ui.interact(close_rect, ui.id().with("close"), Sense::click());
            ui.painter().circle_filled(
                close_rect.center(),
                12.5,
                Color32::from_rgba_unmultiplied(243, 250, 251, 240),
            );
            art::close_icon(ui.painter(), close_rect);
            if close.clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            close.on_hover_text(self.text("退出挂件", "Quit BongoDSH"));

            let hint = art::hint_bounds(bounds);
            ui.painter().rect_filled(
                hint,
                13.0,
                Color32::from_rgba_unmultiplied(243, 250, 251, 240),
            );
            let (message, color) = if self.paused {
                (
                    self.text("已暂停 · 右键继续", "Paused · right-click to resume"),
                    Color32::from_rgb(190, 131, 112),
                )
            } else if self.demo.is_some() {
                (
                    self.text("双手动作演示中", "Playing hand animation"),
                    Color32::from_rgb(63, 144, 146),
                )
            } else if self.focused {
                (
                    self.text("按键试试 · 右键菜单", "Type here · right-click menu"),
                    Color32::from_rgb(63, 144, 146),
                )
            } else {
                (self.docking.status(self.chinese_font), art::INK)
            };
            ui.painter()
                .circle_filled(pos2(hint.min.x + 13.0, hint.center().y), 3.0, color);
            ui.painter().text(
                hint.center() + vec2(5.0, 0.0),
                Align2::CENTER_CENTER,
                message,
                FontId::proportional(12.0),
                art::INK,
            );
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

fn install_system_font(ctx: &egui::Context) -> bool {
    #[cfg(target_os = "macos")]
    let paths = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
    ];
    #[cfg(target_os = "windows")]
    let paths = {
        let windows = std::env::var_os("WINDIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("C:/Windows"));
        [
            windows.join("Fonts/msyh.ttc"),
            windows.join("Fonts/simhei.ttf"),
        ]
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let paths: [&str; 0] = [];

    let Some(bytes) = paths.iter().find_map(|path| std::fs::read(path).ok()) else {
        return false;
    };
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("system-cjk".to_owned(), FontData::from_owned(bytes).into());
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push("system-cjk".to_owned());
    ctx.set_fonts(fonts);
    true
}
