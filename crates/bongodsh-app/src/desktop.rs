use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, IconData, Rect, Sense,
    ViewportCommand, WindowLevel, pos2, vec2,
};

use bongodsh_core::{InputEvent, TypingState, animation::HandAnimation};

use crate::{
    art,
    detached::{self, Launch, Settings},
    docking::Docking,
    global_input::{GlobalInput, Status},
    input,
    preferences::{self, InputSource},
    sprites::Artwork,
};

mod controls;
mod settings;

pub(crate) fn run(launch: Option<Launch>) -> eframe::Result {
    let size = launch.map_or([300.0, 270.0], |launch| {
        [launch.size, launch.size * 252.0 / 300.0]
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BongoDSH")
            .with_inner_size(size)
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_has_shadow(false)
            .with_always_on_top()
            .with_active(launch.is_none())
            .with_visible(launch.is_none())
            .with_icon(IconData::default()),
        renderer: eframe::Renderer::Glow,
        centered: launch.is_none(),
        ..Default::default()
    };
    eframe::run_native(
        "BongoDSH",
        options,
        Box::new(move |creation| Ok(Box::new(Desktop::new(creation, launch)?))),
    )
}

struct Desktop {
    artwork: Option<Artwork>,
    docking: Docking,
    parent: Option<std::sync::mpsc::Receiver<()>>,
    ready: bool,
    character: usize,
    character_size: f32,
    hand_amplitude: u16,
    body_amplitude: u16,
    last_sent_settings: Option<Settings>,
    bridge_failed: bool,
    typing: TypingState,
    animation: HandAnimation,
    last_frame: Instant,
    paused: bool,
    focused: bool,
    always_on_top: bool,
    chinese_font: bool,
    source: InputSource,
    global: GlobalInput,
    keyboard_settings_open: bool,
    desktop_settings_open: bool,
    desktop_settings_focused: bool,
    desktop_settings_size: egui::Vec2,
    character_error: Option<String>,
    settings_error: Option<String>,
    preferences_path: Option<std::path::PathBuf>,
    preferences_error: Option<String>,
}

impl Desktop {
    fn new(
        creation: &eframe::CreationContext<'_>,
        launch: Option<Launch>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ctx = &creation.egui_ctx;
        let artwork = launch
            .map(|launch| Artwork::load(ctx, launch.character))
            .transpose()
            .map_err(std::io::Error::other)?;
        let docking = Docking::new(creation, launch.map(|launch| launch.grab));
        if launch.is_some() && !docking.available() {
            return Err(std::io::Error::other("当前平台无法接续桌面拖动。").into());
        }
        let parent = launch
            .map(|_| detached::watch_parent(ctx.clone()))
            .transpose()?;
        let (preferences_path, source, preferences_error) = match preferences::path() {
            Ok(path) => {
                let (source, error) = match preferences::load(&path) {
                    Ok(source) => (source, None),
                    Err(error) => (
                        InputSource::Window,
                        Some(format!("键盘输入偏好读取失败，已使用窗口输入：{error}。")),
                    ),
                };
                (Some(path), source, error)
            }
            Err(error) => (
                None,
                InputSource::Window,
                Some(format!("键盘输入偏好读取失败，已使用窗口输入：{error}。")),
            ),
        };
        if launch.is_some() {
            ctx.set_theme(egui::ThemePreference::System);
        } else {
            ctx.set_visuals(egui::Visuals::light());
            ctx.global_style_mut(|style| {
                style.visuals.window_fill = Color32::from_rgb(243, 250, 251);
                style.visuals.override_text_color = Some(art::INK);
                style.visuals.selection.bg_fill = Color32::from_rgb(162, 215, 221);
                style.spacing.button_padding = vec2(10.0, 7.0);
            });
        }
        let mut app = Self {
            artwork,
            docking,
            parent,
            ready: false,
            character: launch.map_or(0, |launch| launch.character),
            character_size: launch.map_or(160.0, |launch| launch.size),
            hand_amplitude: launch.map_or(100, |launch| launch.hand_amplitude),
            body_amplitude: launch.map_or(100, |launch| launch.body_amplitude),
            last_sent_settings: None,
            bridge_failed: false,
            typing: TypingState::default(),
            animation: HandAnimation::default(),
            last_frame: Instant::now(),
            paused: launch.is_some_and(|launch| launch.paused),
            focused: false,
            always_on_top: true,
            chinese_font: install_system_font(ctx),
            source,
            global: GlobalInput::default(),
            keyboard_settings_open: false,
            desktop_settings_open: false,
            desktop_settings_focused: false,
            desktop_settings_size: vec2(264.0, 410.0),
            character_error: None,
            settings_error: None,
            preferences_path,
            preferences_error,
        };
        app.restart_input(ctx);
        Ok(app)
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

    fn restart_input(&mut self, ctx: &egui::Context) {
        self.global.stop();
        self.reset();
        if self.source.should_listen_globally(self.paused) {
            self.global.start(ctx);
        }
    }

    fn switch_source(&mut self, source: InputSource, ctx: &egui::Context) {
        if source == self.source {
            self.restart_input(ctx);
            return;
        }
        self.source = source;
        self.preferences_error = match &self.preferences_path {
            Some(path) => preferences::save(path, source).err().map(|error| {
                format!("键盘输入偏好保存失败，本次选择仍生效，重启后可能恢复旧设置：{error}。")
            }),
            None => Some(
                "键盘输入偏好保存失败，无法确定本机设置目录；本次选择仅在当前运行中有效。"
                    .to_owned(),
            ),
        };
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
        ui.separator();
        ui.label(self.text("键盘输入", "Keyboard input"));
        let mut source = self.source;
        ui.horizontal_wrapped(|ui| {
            ui.radio_value(
                &mut source,
                InputSource::Window,
                self.text("窗口输入", "Window"),
            );
            ui.radio_value(
                &mut source,
                InputSource::Global,
                self.text("全局键盘", "Global"),
            )
            .on_hover_text(self.text(
                "在其他应用打字时也响应，不读取输入文字。",
                "Responds while typing in other apps without reading typed text.",
            ));
        });
        if source != self.source {
            self.switch_source(source, ui.ctx());
        }
        if let Some(error) = &self.preferences_error {
            ui.colored_label(Color32::from_rgb(160, 76, 60), error);
        } else {
            ui.small(self.text("重启后恢复此选择。", "Restored after restart."));
        }
        if self.paused {
            ui.small(self.text("键盘响应已暂停", "Keyboard response paused"));
        } else if self.source == InputSource::Global {
            ui.small(self.status_text());
        }
        if let Status::Failed(error) = self.global.status() {
            ui.colored_label(Color32::from_rgb(160, 76, 60), error);
        }
        #[cfg(target_os = "macos")]
        if self.source == InputSource::Global {
            if matches!(
                self.global.status(),
                Status::PermissionRequired | Status::Failed(_)
            ) {
                ui.small(self.text(
                    "请在系统“输入监控”中授权，必要时重启挂件。",
                    "Allow Input Monitoring, then restart if needed.",
                ))
                .on_hover_text(self.text(
                    "通过终端或编辑器启动时，系统可能列出启动它的应用。",
                    "When launched from a terminal or editor, macOS may list that app.",
                ));
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(self.text("授权输入监控", "Request access"))
                    .clicked()
                {
                    crate::global_input::request_permission();
                    self.switch_source(InputSource::Global, ui.ctx());
                }
                if ui
                    .button(self.text("系统设置", "System Settings"))
                    .clicked()
                {
                    self.settings_error = crate::global_input::open_permission_settings().err();
                }
            });
        }
        if self.source == InputSource::Global
            && ui.button(self.text("重新连接", "Reconnect")).clicked()
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

    fn publish_settings(&mut self, ctx: Option<&egui::Context>) {
        if !self.ready || self.parent.is_none() || self.bridge_failed {
            return;
        }
        let settings = Settings {
            character: self.character,
            size: self.character_size,
            paused: self.paused,
            hand_amplitude: self.hand_amplitude,
            body_amplitude: self.body_amplitude,
        };
        if self.last_sent_settings == Some(settings) {
            return;
        }
        if settings.write_to(&mut std::io::stdout().lock()).is_ok() {
            self.last_sent_settings = Some(settings);
        } else {
            self.bridge_failed = true;
            if let Some(ctx) = ctx {
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, ViewportCommand::Close);
            }
        }
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
        let pause_label = if self.paused {
            self.text("继续动画", "Resume")
        } else {
            self.text("暂停动画", "Pause")
        };
        if ui.button(pause_label).clicked() {
            self.paused = !self.paused;
            self.restart_input(ui.ctx());
            ui.close();
        }
        let top_label = self.text("保持置顶", "Always on top");
        if ui.checkbox(&mut self.always_on_top, top_label).changed() {
            ui.ctx().send_viewport_cmd_to(
                egui::ViewportId::ROOT,
                ViewportCommand::WindowLevel(if self.always_on_top {
                    WindowLevel::AlwaysOnTop
                } else {
                    WindowLevel::Normal
                }),
            );
        }
        self.docking.menu(ui, self.chinese_font);
        ui.separator();
        let close_label = if self.parent.is_some() {
            self.text("返回 DSH", "Return to DSH")
        } else {
            self.text("退出挂件", "Quit BongoDSH")
        };
        if ui.button(close_label).clicked() {
            ui.ctx()
                .send_viewport_cmd_to(egui::ViewportId::ROOT, ViewportCommand::Close);
        }
    }
}

impl eframe::App for Desktop {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(parent) = &self.parent
            && matches!(
                parent.try_recv(),
                Ok(()) | Err(std::sync::mpsc::TryRecvError::Disconnected)
            )
        {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }
        let now = Instant::now();
        let settings_active =
            self.parent.is_some() && (self.desktop_settings_open || self.keyboard_settings_open);
        let events = self.global.poll();
        if !settings_active && !self.paused {
            for event in events {
                self.apply(event);
            }
        }
        let local_input = !self.global_selected();
        ctx.input(|state| {
            if local_input && !state.focused && self.focused {
                self.reset();
            }
            self.focused = state.focused;
            if !settings_active && local_input && state.focused && !self.paused {
                for event in &state.events {
                    if let Some(input) = input::normalize(event) {
                        self.apply(input);
                    }
                }
            }
        });
        if self
            .animation
            .advance(now.saturating_duration_since(self.last_frame))
        {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        self.last_frame = now;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bounds = ui.max_rect();
        let hit = self.artwork.as_ref().map_or_else(
            || art::hit_bounds(bounds),
            |artwork| artwork.content_bounds(bounds),
        );
        self.docking.update(ui.ctx(), hit);
        let response = ui.interact(
            hit,
            ui.id().with("character"),
            // Typing Enter/Space must animate hands, not activate a focused button.
            Sense::CLICK | Sense::DRAG,
        );
        if response.drag_started_by(egui::PointerButton::Primary) {
            self.docking.begin_drag();
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if self.artwork.is_some() {
            if response.secondary_clicked() {
                self.reset();
                self.desktop_settings_focused = false;
                self.desktop_settings_open = true;
            }
        } else {
            response.context_menu(|ui| self.menu(ui));
        }
        let response = response.on_hover_cursor(egui::CursorIcon::Grab);
        if let Some(artwork) = &self.artwork {
            let mut painter = ui.painter().clone();
            if self.paused {
                painter.set_opacity(0.62);
            }
            artwork.draw(
                &painter,
                bounds,
                self.animation.positions(),
                self.hand_amplitude,
                self.body_amplitude,
            );
        } else {
            art::draw(ui.painter(), bounds, self.animation.positions());
        }
        if !self.ready && self.parent.is_some() {
            use std::io::Write;
            self.ready = true;
            ui.ctx().send_viewport_cmd(ViewportCommand::Visible(true));
            let mut output = std::io::stdout().lock();
            if writeln!(output, "{{\"event\":\"ready\"}}")
                .and_then(|_| output.flush())
                .is_err()
            {
                self.bridge_failed = true;
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
        }

        if self.artwork.is_none() && (response.hovered() || self.focused || self.paused) {
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
                    self.text("窗口输入 · 右键菜单", "Window input · right-click menu"),
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
        self.desktop_window(ui.ctx());
        self.publish_settings(Some(ui.ctx()));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.publish_settings(None);
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
