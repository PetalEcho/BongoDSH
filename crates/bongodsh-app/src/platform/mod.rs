//! OS objects stay on their owning threads; only geometry crosses the channel.

use std::sync::mpsc::{self, Receiver, SyncSender};

use eframe::egui::{Context, Rect};

use crate::positioning::{AttachmentKind, Monitor, TargetWindow, Widget, WindowId};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
pub(crate) use macos::NativeWindow;
#[cfg(target_os = "windows")]
pub(crate) use windows::NativeWindow;

#[cfg(target_os = "macos")]
use macos::Probe;
#[cfg(target_os = "windows")]
use windows::Probe;

pub(crate) struct Request {
    pub generation: u64,
    pub widget: Widget,
    pub monitors: Vec<Monitor>,
    pub snap: bool,
    pub inputs: bool,
    pub follow: Option<(WindowId, AttachmentKind)>,
}

pub(crate) struct Snapshot {
    pub generation: u64,
    pub snap: bool,
    pub windows: Vec<TargetWindow>,
    pub input: Option<(WindowId, Rect)>,
    pub input_access: bool,
}

pub(crate) struct Observer {
    requests: SyncSender<Request>,
    replies: Receiver<Snapshot>,
}

impl Observer {
    pub fn new(ctx: Context) -> std::io::Result<Self> {
        let (requests, incoming) = mpsc::sync_channel::<Request>(1);
        let (outgoing, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("window-geometry".into())
            .spawn(move || {
                let mut probe = Probe::new();
                while let Ok(request) = incoming.recv() {
                    let snapshot = probe.sample(&request);
                    if outgoing.send(snapshot).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })?;
        Ok(Self { requests, replies })
    }

    pub fn request(&self, request: Request) -> bool {
        self.requests.try_send(request).is_ok()
    }

    pub fn receive(&self) -> Option<Snapshot> {
        self.replies.try_recv().ok()
    }
}
