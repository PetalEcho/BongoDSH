use std::{
    ffi::OsString,
    io::{BufRead, BufReader, Read, Write},
    sync::mpsc::{self, Receiver},
};

use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Launch {
    pub character: usize,
    pub size: f32,
    pub grab: egui::Vec2,
    pub paused: bool,
    pub hand_amplitude: u16,
    pub body_amplitude: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub character: usize,
    pub size: f32,
    pub paused: bool,
    pub hand_amplitude: u16,
    pub body_amplitude: u16,
}

impl Settings {
    pub fn write_to(&self, output: &mut impl Write) -> std::io::Result<()> {
        let character = CHARACTER_IDS
            .get(self.character)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "未知角色。"))?;
        if !self.size.is_finite()
            || self.size.fract() != 0.0
            || !(100.0..=220.0).contains(&self.size)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "角色大小无效。",
            ));
        }
        if self.hand_amplitude > 200 || self.body_amplitude > 200 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "动作幅度应为 0–200 的整数百分比。",
            ));
        }
        writeln!(
            output,
            "{{\"event\":\"settings\",\"character\":\"{character}\",\"size\":{},\"paused\":{},\"handAmplitude\":{},\"bodyAmplitude\":{}}}",
            self.size, self.paused, self.hand_amplitude, self.body_amplitude
        )?;
        output.flush()
    }
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
        if size.fract() != 0.0 {
            return Err("角色大小应为 100–220 的整数。");
        }
        let mut grab = None;
        let mut paused = None;
        let mut hand_amplitude = None;
        let mut body_amplitude = None;
        let mut rest = rest;
        while !rest.is_empty() {
            match rest {
                [flag, x, y, remaining @ ..] if flag == "--grab" && grab.is_none() => {
                    grab = Some(egui::vec2(number(x, 0.0, 1.0)?, number(y, 0.0, 1.0)?));
                    rest = remaining;
                }
                [flag, value, remaining @ ..] if flag == "--paused" && paused.is_none() => {
                    paused = Some(match value.to_str() {
                        Some("true") => true,
                        Some("false") => false,
                        _ => return Err("暂停参数应为 --paused true 或 --paused false。"),
                    });
                    rest = remaining;
                }
                [flag, value, remaining @ ..]
                    if flag == "--hand-amplitude" && hand_amplitude.is_none() =>
                {
                    hand_amplitude = Some(percentage(value)?);
                    rest = remaining;
                }
                [flag, value, remaining @ ..]
                    if flag == "--body-amplitude" && body_amplitude.is_none() =>
                {
                    body_amplitude = Some(percentage(value)?);
                    rest = remaining;
                }
                _ => {
                    return Err(
                        "可选参数为 --grab、--paused、--hand-amplitude 和 --body-amplitude，各使用一次。",
                    );
                }
            }
        }
        Ok(Self {
            character: character?,
            size,
            grab: grab.unwrap_or(egui::Vec2::splat(0.5)),
            paused: paused.unwrap_or(false),
            hand_amplitude: hand_amplitude.unwrap_or(100),
            body_amplitude: body_amplitude.unwrap_or(100),
        })
    }
}

fn percentage(value: &OsString) -> Result<u16, &'static str> {
    value
        .to_str()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value <= 200)
        .ok_or("动作幅度应为 0–200 的整数百分比。")
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
