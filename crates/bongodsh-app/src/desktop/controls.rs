use eframe::egui::{self, Align, Layout, vec2};

pub(super) fn size_slider(ui: &mut egui::Ui, size: &mut f32) -> egui::Response {
    ui.add(
        egui::Slider::new(size, 100.0..=220.0)
            .integer()
            // integer() resets the step to one, so apply the page's ten-pixel step last.
            .step_by(10.0)
            .show_value(false),
    )
}

pub(super) fn amplitude_slider(ui: &mut egui::Ui, percent: &mut u16) -> egui::Response {
    ui.add(
        egui::Slider::new(percent, 0..=200)
            .integer()
            .step_by(10.0)
            .show_value(false),
    )
}

pub(super) fn checkbox_row(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
    ui.push_id(label, |ui| {
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), 30.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.label(label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.scope(|ui| {
                        let widgets = &mut ui.visuals_mut().widgets;
                        for widget in [
                            &mut widgets.inactive,
                            &mut widgets.hovered,
                            &mut widgets.active,
                        ] {
                            widget.corner_radius = 2.into();
                        }
                        ui.add(egui::Checkbox::without_text(value)).changed()
                    })
                    .inner
                })
                .inner
            },
        )
        .inner
    })
    .inner
}
