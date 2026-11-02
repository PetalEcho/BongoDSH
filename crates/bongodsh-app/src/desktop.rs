use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, IconData, Rect, Sense,
    ViewportCommand, WindowLevel, pos2, vec2,
};

use bongodsh_core::{InputEvent, KeyId, TypingState, animation::HandAnimation};

use crate::{
    appearance::{self, Appearance},
    appearance_ui::{self, AppearanceUi},
    art,
    global_input::{GlobalInput, Status},
    input,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum InputSource {
    #[default]
    Window,
    Global,
}

pub(crate) fn run(open_appearance: bool) -> eframe::Result {
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
        Box::new(move |creation| {
            let mut desktop = Desktop::new(&creation.egui_ctx);
            desktop.appearance_ui.open = open_appearance;
            Ok(Box::new(desktop))
        }),
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
    source: InputSource,
    global: GlobalInput,
    keyboard_settings_open: bool,
    settings_error: Option<String>,
    appearance: Appearance,
    appearance_ui: AppearanceUi,
}

impl Desktop {
    fn new(ctx: &egui::Context) -> Self {
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
            source: InputSource::Window,
            global: GlobalInput::default(),
            keyboard_settings_open: false,
            settings_error: None,
            appearance: Appearance::new(ctx, appearance::data_path()),
            appearance_ui: AppearanceUi::default(),
        }
    }

    fn text(&self, chinese: &'static str, english: &'static str) -> &'static str {
        if self.chinese_font { chinese } else { english }
    }

    fn apply(&mut self, event: InputEvent) {
        if event == InputEvent::Reset {
            self.reset();
            return;
        }
        self.animation.apply(self.typing.apply(event));
    }

    fn reset(&mut self) {
        self.typing.apply(InputEvent::Reset);
        self.animation = HandAnimation::default();
    }

    fn start_demo(&mut self) {
        self.global.stop();
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
        if demo.remaining == 0 {
            self.demo = None;
            self.restart_input(ctx);
            return;
        }
        let key = KeyId::new(u32::from(demo.remaining));
        demo.remaining -= 1;
        demo.next_at = now + Duration::from_millis(190);
        self.apply(InputEvent::Press { key, repeat: false });
        self.apply(InputEvent::Release(key));
        ctx.request_repaint_after(Duration::from_millis(190));
    }

    fn restart_input(&mut self, ctx: &egui::Context) {
        self.global.stop();
        self.reset();
        if self.source == InputSource::Global && !self.paused && self.demo.is_none() {
            self.global.start(ctx);
        }
    }

    fn switch_source(&mut self, source: InputSource, ctx: &egui::Context) {
        self.demo = None;
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

    fn status_text(&self) -> &'static str {
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
        if self.paused || self.demo.is_some() {
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
            ui.label(self.text("通过终端或编辑器启动时，系统可能列出启动它的应用。授权后若仍不可用，请退出并重新启动挂件。", "When launched from a terminal or editor, macOS may list that app. If capture remains unavailable after granting access, quit and relaunch BongoDSH."));
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
            egui::ViewportId::from_hash_of("keyboard-settings"),
            egui::ViewportBuilder::default()
                .with_title(self.text("BongoDSH · 键盘输入", "BongoDSH · Keyboard input"))
                .with_inner_size([420.0, 380.0])
                .with_min_inner_size([380.0, 340.0]),
            |ui, _class| {
                if ui.ctx().input(|input| input.viewport().close_requested()) {
                    self.keyboard_settings_open = false;
                }
                egui::Frame::central_panel(ui.style())
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| self.keyboard_panel(ui));
                    });
            },
        );
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(185.0);
        ui.label(egui::RichText::new("BongoDSH").strong());
        ui.label(self.status_text());
        ui.separator();
        if ui
            .button(self.text("键盘输入设置…", "Keyboard input settings…"))
            .clicked()
        {
            self.keyboard_settings_open = true;
            ui.close();
        }
        if !self.global_selected()
            && ui
                .button(self.text("启用全局键盘", "Enable global keyboard"))
                .clicked()
        {
            self.switch_source(InputSource::Global, ui.ctx());
            self.keyboard_settings_open = true;
            ui.close();
        }
        if ui
            .button(self.text("角色与图片…", "Character & images…"))
            .clicked()
        {
            self.appearance_ui.open = true;
            self.demo = None;
            self.restart_input(ui.ctx());
            ui.ctx().send_viewport_cmd_to(
                egui::ViewportId::from_hash_of(appearance_ui::VIEWPORT),
                ViewportCommand::Focus,
            );
            ui.close();
        }
        if ui
            .button(self.text("演示按键动作", "Play typing animation"))
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
            self.restart_input(ui.ctx());
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
        ui.separator();
        if ui.button(self.text("退出挂件", "Quit BongoDSH")).clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

impl eframe::App for Desktop {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        if self.appearance.poll(ctx) && self.appearance_ui.open {
            ctx.request_repaint_of(egui::ViewportId::from_hash_of(appearance_ui::VIEWPORT));
        }
        if self.appearance_ui.dropped(ctx, &mut self.appearance) {
            self.demo = None;
            self.restart_input(ctx);
        }
        let events = self.global.poll();
        if !self.paused && self.demo.is_none() {
            for event in events {
                self.apply(event);
            }
        }
        let local_input = !self.global_selected() && self.demo.is_none();
        ctx.input(|state| {
            if local_input && !state.focused && self.focused {
                self.reset();
            }
            self.focused = state.focused;
            if local_input && state.focused && !self.paused {
                for event in &state.events {
                    if let Some(input) = input::normalize(event) {
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
            if self.appearance_ui.open {
                ctx.request_repaint_after_for(
                    Duration::from_millis(16),
                    egui::ViewportId::from_hash_of(appearance_ui::VIEWPORT),
                );
            }
        }
        self.last_frame = now;
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let bounds = ui.max_rect();
        let response = ui.interact(
            if self.appearance.layers()[0].is_some() {
                Rect::from_min_max(
                    bounds.min + vec2(23.0, 20.0),
                    bounds.min + vec2(277.0, 219.0),
                )
            } else {
                art::hit_bounds(bounds)
            },
            ui.id().with("character"),
            // Typing Enter/Space must animate hands, not activate a focused button.
            Sense::CLICK | Sense::DRAG,
        );
        if response.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        response.context_menu(|ui| self.menu(ui));
        let response = response.on_hover_cursor(egui::CursorIcon::Grab);
        art::draw_character(
            ui.painter(),
            bounds,
            self.animation.positions(),
            self.appearance.layers(),
        );

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
                    self.text("按键动作演示中", "Playing typing animation"),
                    Color32::from_rgb(63, 144, 146),
                )
            } else if self.global_selected() {
                (
                    self.text(
                        "全局键盘 · 右键设置",
                        "Global keyboard · right-click settings",
                    ),
                    Color32::from_rgb(63, 144, 146),
                )
            } else if self.focused {
                (
                    self.text("按键试试 · 右键菜单", "Type here · right-click menu"),
                    Color32::from_rgb(63, 144, 146),
                )
            } else {
                (
                    self.text("点击后打字 · 拖动移动", "Click to type · drag to move"),
                    art::INK,
                )
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
        self.keyboard_window(ui.ctx());
        let ctx = ui.ctx().clone();
        self.appearance_ui.show(
            &ctx,
            &mut self.appearance,
            self.animation.positions(),
            self.chinese_font,
        );
        if std::mem::take(&mut self.appearance_ui.play) {
            self.start_demo();
            ctx.request_repaint();
        }
        if let Some(part) = self
            .appearance_ui
            .pick
            .take()
            .filter(|_| !self.appearance.busy())
        {
            self.demo = None;
            self.global.stop();
            self.reset();
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            if let Some(path) = rfd::FileDialog::new()
                .set_title(self.text("选择角色图层", "Choose a character layer"))
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .set_parent(frame)
                .pick_file()
            {
                self.appearance.import(path, part, &ctx);
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            let _ = (frame, part);
            self.restart_input(&ctx);
            self.last_frame = Instant::now();
            ctx.send_viewport_cmd_to(
                egui::ViewportId::from_hash_of(appearance_ui::VIEWPORT),
                ViewportCommand::Focus,
            );
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.global.stop();
        if let Err(problem) = self.appearance.flush() {
            eprintln!("{}", problem.message(self.chinese_font));
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
