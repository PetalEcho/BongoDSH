//! Window probes never request titles, accessibility values, or screenshots.

use std::sync::mpsc::{self, Receiver, SyncSender};

use eframe::egui::Context;

use crate::positioning::TargetWindow;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) use macos::NativeWindow;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) use unsupported::NativeWindow;
#[cfg(target_os = "windows")]
pub(crate) use windows::NativeWindow;

#[cfg(target_os = "macos")]
use macos::visible_windows;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use unsupported::visible_windows;
#[cfg(target_os = "windows")]
use windows::visible_windows;

pub(crate) struct Snapshot {
    pub generation: u64,
    pub windows: Vec<TargetWindow>,
}

pub(crate) struct Observer {
    requests: SyncSender<u64>,
    replies: Receiver<Snapshot>,
}

impl Observer {
    pub fn new(ctx: Context) -> std::io::Result<Self> {
        let (requests, incoming) = mpsc::sync_channel(1);
        let (outgoing, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("window-geometry".into())
            .spawn(move || {
                while let Ok(generation) = incoming.recv() {
                    if outgoing
                        .send(Snapshot {
                            generation,
                            windows: visible_windows(),
                        })
                        .is_err()
                    {
                        break;
                    }
                    ctx.request_repaint();
                }
            })?;
        Ok(Self { requests, replies })
    }

    pub fn request(&self, generation: u64) -> bool {
        self.requests.try_send(generation).is_ok()
    }

    pub fn receive(&self) -> Option<Snapshot> {
        self.replies.try_recv().ok()
    }
}
