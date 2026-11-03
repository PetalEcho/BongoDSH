use std::{
    ffi::OsString,
    io::{BufRead, BufReader, Read},
    sync::mpsc::{self, Receiver},
};

use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Launch {
    pub character: usize,
    pub size: f32,
    pub grab: egui::Vec2,
}

pub(crate) const CHARACTER_IDS: [&str; 6] = [
    "fish-maid",
    "sailor",
    "pajamas",
    "wizard",
    "blue-maid",
    "orange-bride",
];

impl Launch {
    pub fn parse(arguments: &[OsString]) -> Result<Self, &'static str> {
        let [id, size, rest @ ..] = arguments else {
            return Err("桌面模式需要角色和大小参数。");
        };
        let character = CHARACTER_IDS
            .iter()
            .position(|candidate| id == *candidate)
            .ok_or("未知角色。");
        let size = number(size, 100.0, 220.0)?;
        let grab = match rest {
            [] => egui::Vec2::splat(0.5),
            [flag, x, y] if flag == "--grab" => {
                egui::vec2(number(x, 0.0, 1.0)?, number(y, 0.0, 1.0)?)
            }
            _ => return Err("抓取位置参数应为 --grab <0..1> <0..1>。"),
        };
        Ok(Self {
            character: character?,
            size,
            grab,
        })
    }
}

fn number(value: &OsString, min: f32, max: f32) -> Result<f32, &'static str> {
    value
        .to_str()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite() && (min..=max).contains(value))
        .ok_or("桌面模式参数超出允许范围。")
}

/// The parent owns this process. A closed pipe must not leave an orphan overlay.
pub(crate) fn watch_parent(ctx: egui::Context) -> std::io::Result<Receiver<()>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("desktop-parent".into())
        .spawn(move || {
            let mut reader = BufReader::new(std::io::stdin().lock());
            loop {
                let mut line = Vec::new();
                let result = reader.by_ref().take(129).read_until(b'\n', &mut line);
                if result.is_err()
                    || line.is_empty()
                    || line.len() > 128
                    || line == b"{\"type\":\"close\"}\n"
                    || line == b"{\"type\":\"close\"}\r\n"
                {
                    let _ = sender.send(());
                    ctx.request_repaint();
                    break;
                }
            }
        })?;
    Ok(receiver)
}
