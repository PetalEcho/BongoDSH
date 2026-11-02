use std::time::Instant;

use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, IconData, Rect, Sense, ViewportCommand,
    ViewportId, WindowLevel, pos2, vec2,
};

use bongodsh_core::{Hand, InputEvent};

use crate::{
    art,
    global_input::{GlobalInput, Status},
    input,
    interaction::Interaction,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum InputSource {
    #[default]
    Window,
    Global,
}

const MIN_SCALE: f32 = 0.7;
const MAX_SCALE: f32 = 1.5;

pub(crate) fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BongoDSH")
            .with_inner_size(widget_size(1.0))
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
        Box::new(|creation| Ok(Box::new(Desktop::new(&creation.egui_ctx)))),
    )
}

pub(crate) fn widget_size(scale: f32) -> egui::Vec2 {
    vec2(
        (300.0 * scale).max(260.0),
        (234.0 * scale + 36.0).max(240.0),
    )
}

struct Desktop {
    interaction: Interaction,
    last_frame: Instant,
    focused: bool,
    always_on_top: bool,
    locked: bool,
    scale: f32,
    travel: f32,
    settings_open: bool,
    menu_open: bool,
    chinese_font: bool,
    source: InputSource,
    global: GlobalInput,
    keyboard_settings_open: bool,
    settings_error: Option<String>,
}

impl Desktop {
    fn new(ctx: &egui::Context) -> Self {
        ctx.set_visuals(egui::Visuals::light());
        ctx.options_mut(|options| options.zoom_with_keyboard = false);
        ctx.global_style_mut(|style| {
            style.visuals.window_fill = Color32::from_rgb(243, 250, 251);
            style.visuals.panel_fill = Color32::from_rgb(243, 250, 251);
            style.visuals.override_text_color = Some(art::INK);
            style.visuals.selection.bg_fill = Color32::from_rgb(162, 215, 221);
            style.spacing.button_padding = vec2(10.0, 5.0);
        });
        Self {
            interaction: Interaction::default(),
            last_frame: Instant::now(),
            focused: false,
            always_on_top: true,
            locked: false,
            scale: 1.0,
            travel: 1.0,
            settings_open: false,
            menu_open: false,
            chinese_font: install_system_font(ctx),
            source: InputSource::Window,
            global: GlobalInput::default(),
            keyboard_settings_open: false,
            settings_error: None,
        }
    }

    fn text(&self, chinese: &'static str, english: &'static str) -> &'static str {
        if self.chinese_font { chinese } else { english }
    }

    fn open_settings(&mut self, ctx: &egui::Context) {
        self.cancel_preview(ctx);
        self.settings_open = true;
        ctx.send_viewport_cmd_to(settings_id(), ViewportCommand::Focus);
    }

    fn open_keyboard_settings(&mut self, ctx: &egui::Context) {
        self.keyboard_settings_open = true;
        ctx.send_viewport_cmd_to(keyboard_settings_id(), ViewportCommand::Focus);
    }

    fn resize(&mut self, ctx: &egui::Context, scale: f32) {
        let scale = scale.clamp(MIN_SCALE, MAX_SCALE);
        if (scale - self.scale).abs() < 0.001 {
            return;
        }
        let size = widget_size(scale);
        // Keep the feet at the same screen position as the character changes size.
        if let Some(rect) = ctx.input(|state| {
            state
                .raw
                .viewports
                .get(&ViewportId::ROOT)
                .and_then(|viewport| viewport.inner_rect)
        }) {
            let anchor = rect.min + vec2(rect.width() * 0.5, 218.0 * self.scale);
            let position = anchor - vec2(size.x * 0.5, 218.0 * scale);
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::OuterPosition(position));
        }
        self.scale = scale;
        ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::InnerSize(size));
    }

    fn update_window_level(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd_to(
            ViewportId::ROOT,
            ViewportCommand::WindowLevel(if self.always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            }),
        );
    }

    fn restart_input(&mut self, ctx: &egui::Context) {
        self.global.stop();
        self.interaction.reset();
        if self.source == InputSource::Global && !self.interaction.paused {
            self.global.start(ctx);
        }
    }

    fn cancel_preview(&mut self, ctx: &egui::Context) {
        if self.source == InputSource::Global && self.interaction.is_previewing() {
            self.restart_input(ctx);
        } else {
            self.interaction.reset();
        }
    }

    fn switch_source(&mut self, source: InputSource, ctx: &egui::Context) {
        self.source = source;
        self.restart_input(ctx);
    }

    fn global_selected(&self) -> bool {
        self.source == InputSource::Global
            && matches!(
                self.global.status(),
                Status::Starting | Status::Listening | Status::Suspended
            )
    }

    fn tap(&mut self, hand: Option<Hand>) {
        if !self.interaction.paused {
            self.global.stop();
            self.interaction.tap(hand);
        }
    }

    fn start_demo(&mut self) {
        if !self.interaction.paused {
            self.global.stop();
            self.interaction.start_demo(Instant::now());
        }
    }

    fn toggle_pause(&mut self, ctx: &egui::Context) {
        self.interaction.toggle_pause();
        self.restart_input(ctx);
    }

    fn status_text(&self) -> &'static str {
        if self.interaction.paused {
            return self.text("键盘反馈已暂停", "Keyboard feedback paused");
        }
        if self.interaction.is_previewing() && !self.interaction.preview_finished() {
            return self.text(
                "动作预览中 · 真实监听暂停",
                "Preview playing · live capture paused",
            );
        }
        match self.global.status() {
            Status::Off => self.text("仅响应本窗口按键", "Keyboard input in this window"),
            Status::Starting => self.text("正在连接全局键盘", "Connecting global keyboard"),
            Status::Listening => self.text("全局键盘已连接", "Global keyboard connected"),
            Status::Suspended => self.text(
                "系统暂时限制键盘监听",
                "Keyboard capture suspended by system",
            ),
            Status::PermissionRequired => self.text(
                "需要输入监控权限 · 暂用窗口输入",
                "Input Monitoring required; using window input",
            ),
            Status::Failed(_) => self.text(
                "全局监听不可用 · 暂用窗口输入",
                "Global capture unavailable; using window input",
            ),
        }
    }

    fn keyboard_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(self.status_text());
        ui.separator();
        let mut source = self.source;
        ui.radio_value(
            &mut source,
            InputSource::Window,
            self.text("仅窗口内输入", "Input in this window"),
        );
        ui.radio_value(
            &mut source,
            InputSource::Global,
            self.text(
                "全局键盘 · 在其他应用打字也响应",
                "Global keyboard, including other apps",
            ),
        );
        if source != self.source {
            self.switch_source(source, ui.ctx());
        }
        ui.add_space(8.0);
        ui.label(self.text("只处理物理按键的按下和松开，不读取、保存或上传输入文字。", "Only physical key presses and releases are processed. Typed text is never read, stored or uploaded."));
        if self.interaction.paused || self.interaction.is_previewing() {
            ui.label(self.text(
                "暂停或演示期间停止真实键盘监听。",
                "Live capture is stopped while paused or playing the demo.",
            ));
        }
        if let Status::Failed(error) = self.global.status() {
            ui.colored_label(Color32::from_rgb(160, 76, 60), error);
        }
        #[cfg(target_os = "macos")]
        {
            ui.separator();
            ui.label(self.text(
                "macOS 需要“隐私与安全性 → 输入监控”授权。",
                "macOS requires Privacy & Security > Input Monitoring.",
            ));
            ui.label(self.text("列表中没有本应用时，点击系统设置中的“+”手动添加。可用下方按钮在访达中定位当前应用。", "If this app is missing from the list, use the + button in System Settings to add it. Reveal the current app in Finder below."));
            ui.label(self.text("开启权限后检查并重连；仍不可用时退出并重新打开挂件。通过终端或编辑器启动时，系统可能列出启动它的应用。", "After granting access, reconnect, or quit and relaunch if needed. macOS may list the terminal or editor when launching an unpackaged binary."));
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(self.text("申请输入监控权限", "Request Input Monitoring"))
                    .clicked()
                {
                    crate::global_input::request_permission();
                    self.switch_source(InputSource::Global, ui.ctx());
                }
                if ui
                    .button(self.text("打开系统设置", "Open System Settings"))
                    .clicked()
                {
                    self.settings_error = crate::global_input::open_permission_settings().err();
                }
                if ui
                    .button(self.text("在访达中显示应用", "Reveal app in Finder"))
                    .clicked()
                {
                    self.settings_error = crate::global_input::reveal_application().err();
                }
            });
        }
        if self.source == InputSource::Global
            && ui
                .button(self.text("检查并重新连接", "Check and reconnect"))
                .clicked()
        {
            self.restart_input(ui.ctx());
        }
        if let Some(error) = &self.settings_error {
            ui.colored_label(Color32::from_rgb(160, 76, 60), error);
        }
    }

    fn keyboard_window(&mut self, ctx: &egui::Context) {
        if !self.keyboard_settings_open {
            return;
        }
        ctx.show_viewport_immediate(
            keyboard_settings_id(),
            egui::ViewportBuilder::default()
                .with_title(self.text("BongoDSH · 键盘输入", "BongoDSH · Keyboard input"))
                .with_inner_size([420.0, 440.0])
                .with_min_inner_size([380.0, 340.0]),
            |ui, _class| {
                if ui.ctx().input(|input| input.viewport().close_requested()) {
                    self.keyboard_settings_open = false;
                    return;
                }
                ui.painter()
                    .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(243, 250, 251));
                egui::Frame::central_panel(ui.style())
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| self.keyboard_panel(ui));
                    });
            },
        );
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(190.0);
        ui.label(egui::RichText::new("BongoDSH").strong());
        ui.separator();
        if ui
            .button(self.text("设置与预览…", "Settings & preview…"))
            .clicked()
        {
            self.open_settings(ui.ctx());
            ui.close();
        }
        if ui
            .button(self.text("键盘输入设置…", "Keyboard input settings…"))
            .clicked()
        {
            self.open_keyboard_settings(ui.ctx());
            ui.close();
        }
        let lock_label = self.text("锁定位置", "Lock position");
        if ui.checkbox(&mut self.locked, lock_label).changed() {
            self.cancel_preview(ui.ctx());
        }
        ui.separator();
        if ui.button(self.text("退出挂件", "Quit BongoDSH")).clicked() {
            ui.ctx()
                .send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
        }
    }

    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        ctx.show_viewport_immediate(
            settings_id(),
            egui::ViewportBuilder::default()
                .with_title(self.text("BongoDSH · 设置", "BongoDSH · Settings"))
                .with_inner_size([330.0, 470.0])
                .with_resizable(false)
                .with_always_on_top(),
            |ui, _class| {
                if ui.input(|state| {
                    state.viewport().close_requested() || state.key_pressed(egui::Key::Escape)
                }) {
                    self.settings_open = false;
                    return;
                }
                ui.painter()
                    .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(243, 250, 251));
                egui::Frame::new()
                    .fill(Color32::from_rgb(243, 250, 251))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(self.text("动作预览", "Animation preview"))
                                .strong(),
                        );
                        let (preview, _) = ui
                            .allocate_exact_size(vec2(ui.available_width(), 142.0), Sense::hover());
                        art::draw(
                            ui.painter(),
                            art::CharacterLayout::new(preview, 0.6),
                            self.interaction.animation.positions(),
                            self.travel,
                        );
                        ui.add_enabled_ui(!self.interaction.paused, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button(self.text("左手", "Left")).clicked() {
                                    self.tap(Some(Hand::Left));
                                }
                                if ui.button(self.text("右手", "Right")).clicked() {
                                    self.tap(Some(Hand::Right));
                                }
                                let label = if self.interaction.is_demo_playing() {
                                    self.text("停止演示", "Stop demo")
                                } else {
                                    self.text("节奏演示", "Play rhythm")
                                };
                                if ui.button(label).clicked() {
                                    if self.interaction.is_demo_playing() {
                                        self.restart_input(ui.ctx());
                                    } else {
                                        self.start_demo();
                                    }
                                }
                            });
                        });
                        ui.add_space(10.0);
                        ui.separator();
                        let mut percent = self.scale * 100.0;
                        if ui
                            .add(
                                egui::Slider::new(&mut percent, 70.0..=150.0)
                                    .suffix("%")
                                    .text(self.text("挂件大小", "Size")),
                            )
                            .changed()
                        {
                            self.resize(ui.ctx(), percent / 100.0);
                        }
                        let travel_label = self.text("敲击幅度", "Hand travel");
                        if ui
                            .add(
                                egui::Slider::new(&mut self.travel, 0.6..=1.3)
                                    .fixed_decimals(1)
                                    .text(travel_label),
                            )
                            .changed()
                        {
                            self.tap(None);
                        }
                        let lock_label = self.text("锁定位置", "Lock position");
                        ui.checkbox(&mut self.locked, lock_label);
                        let top_label = self.text("保持置顶", "Always on top");
                        if ui.checkbox(&mut self.always_on_top, top_label).changed() {
                            self.update_window_level(ui.ctx());
                        }
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let label = if self.interaction.paused {
                                self.text("继续动画", "Resume")
                            } else {
                                self.text("暂停动画", "Pause")
                            };
                            if ui.button(label).clicked() {
                                self.toggle_pause(ui.ctx());
                            }
                            if ui.button(self.text("恢复默认", "Reset defaults")).clicked() {
                                self.resize(ui.ctx(), 1.0);
                                self.travel = 1.0;
                                self.locked = false;
                                self.always_on_top = true;
                                self.update_window_level(ui.ctx());
                                self.interaction.paused = false;
                                self.source = InputSource::Window;
                                self.restart_input(ui.ctx());
                            }
                        });
                        ui.add_space(10.0);
                        ui.small(self.text(
                            "点击试敲 · 拖动移动 · 滚轮缩放",
                            "Click to tap · drag to move · scroll to resize",
                        ));
                        if ui
                            .button(self.text("键盘输入设置…", "Keyboard input settings…"))
                            .clicked()
                        {
                            self.open_keyboard_settings(ui.ctx());
                        }
                        ui.small(self.status_text());
                        ui.small(
                            self.text("设置仅本次运行有效。", "Settings last for this session."),
                        );
                    });
                if let Some(delay) = self.interaction.next_repaint(Instant::now()) {
                    ui.ctx().request_repaint_after(delay);
                }
            },
        );
    }

    fn controls(&mut self, ui: &mut egui::Ui, bounds: Rect) {
        let menu_rect = Rect::from_min_size(
            pos2(bounds.max.x - 37.0, bounds.min.y + 12.0),
            vec2(25.0, 25.0),
        );
        let menu = ui.interact(menu_rect, ui.id().with("settings"), Sense::click());
        ui.painter().circle_filled(
            menu_rect.center(),
            12.5,
            Color32::from_rgba_unmultiplied(243, 250, 251, 240),
        );
        art::menu_icon(ui.painter(), menu_rect);
        if menu.clicked() {
            self.open_settings(ui.ctx());
        }
        menu.on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(self.text("设置与预览", "Settings & preview"));
    }
}

fn settings_id() -> ViewportId {
    ViewportId::from_hash_of("bongodsh-settings")
}

fn keyboard_settings_id() -> ViewportId {
    ViewportId::from_hash_of("keyboard-settings")
}

impl eframe::App for Desktop {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        self.interaction
            .advance(now.saturating_duration_since(self.last_frame), now);
        self.last_frame = now;
        if self.source == InputSource::Global && self.interaction.preview_finished() {
            self.restart_input(ctx);
        }
        let events = self.global.poll();
        if !self.interaction.paused && !self.interaction.is_previewing() {
            for event in events {
                if !self.menu_open || event == InputEvent::Reset {
                    self.interaction.input(event);
                }
            }
        }
        let local_input = !(self.global_selected()
            || self.source == InputSource::Global && self.interaction.is_previewing());
        let keyboard_available = local_input && !self.menu_open && !ctx.egui_wants_keyboard_input();
        ctx.input(|state| {
            if local_input && !state.focused && self.focused {
                self.interaction.lose_focus();
            }
            self.focused = state.focused;
            if state.focused && keyboard_available {
                for event in &state.events {
                    if let Some(input) = input::normalize(event) {
                        self.interaction.input(input);
                    }
                }
            }
        });
        if let Some(delay) = self.interaction.next_repaint(now) {
            ctx.request_repaint_after(delay);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bounds = ui.max_rect();
        let layout = art::CharacterLayout::new(bounds, self.scale);
        let response = ui.interact(
            layout.hit_bounds(),
            ui.id().with("character"),
            Sense::CLICK | Sense::DRAG,
        );
        if !self.locked && response.drag_started_by(egui::PointerButton::Primary) {
            self.cancel_preview(ui.ctx());
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        } else if response.clicked_by(egui::PointerButton::Primary) {
            // Borderless windows may receive a click without becoming the key window.
            if !self.focused && !self.global_selected() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Focus);
            }
            ui.memory_mut(|memory| {
                if let Some(focused) = memory.focused() {
                    memory.surrender_focus(focused);
                }
            });
            let hand = response.interact_pointer_pos().and_then(|point| {
                layout.hand_at(point, self.interaction.animation.positions(), self.travel)
            });
            self.tap(hand);
        }
        if response.secondary_clicked() {
            self.cancel_preview(ui.ctx());
        }
        response.context_menu(|ui| self.menu(ui));
        self.menu_open = response.context_menu_opened();
        let cursor = if self.locked {
            egui::CursorIcon::PointingHand
        } else if response.dragged() {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Grab
        };
        let response = response.on_hover_cursor(cursor);
        if response.hovered() && !self.menu_open && !response.dragged() {
            let scroll = ui.input(|state| state.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                self.resize(ui.ctx(), self.scale * (scroll * 0.002).exp());
            }
        }
        art::draw(
            ui.painter(),
            layout,
            self.interaction.animation.positions(),
            self.travel,
        );
        if ui.input(|state| state.pointer.hover_pos().is_some()) {
            self.controls(ui, bounds);
        }
        self.settings(ui.ctx());
        self.keyboard_window(ui.ctx());
        if let Some(delay) = self.interaction.next_repaint(Instant::now()) {
            ui.ctx().request_repaint_after(delay);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.global.stop();
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
