// Native listeners never receive text. Each session owns its event queue and
// thread; dropping it disables capture, joins the thread and discards old input.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
};

use bongodsh_core::InputEvent;
use eframe::egui;

mod keys;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use unsupported as platform;

const QUEUE_CAPACITY: usize = 256;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Status {
    #[default]
    Off,
    Starting,
    Listening,
    Suspended,
    PermissionRequired,
    Failed(String),
}

/// Only the UI calls permission requests, in response to an explicit click.
#[cfg(target_os = "macos")]
pub(crate) fn request_permission() -> bool {
    platform::request_permission()
}

#[cfg(target_os = "macos")]
pub(crate) fn open_permission_settings() -> Result<(), String> {
    platform::open_permission_settings()
}

#[cfg(target_os = "macos")]
pub(crate) fn reveal_application() -> Result<(), String> {
    platform::reveal_application()
}

#[derive(Clone)]
struct Sink {
    events: SyncSender<InputEvent>,
    statuses: mpsc::Sender<Status>,
    overflow: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    context: egui::Context,
}

impl Sink {
    fn input(&self, event: InputEvent) {
        if self.stopped.load(Ordering::Acquire) || self.overflow.load(Ordering::Acquire) {
            return;
        }
        if let Err(TrySendError::Full(_)) = self.events.try_send(event) {
            // Losing a release must never leave a hand down. The UI discards the
            // entire batch on overflow, then resets before accepting more input.
            self.overflow.store(true, Ordering::Release);
        }
    }

    fn status(&self, status: Status) {
        let _ = self.statuses.send(status);
        self.wake();
    }

    fn wake(&self) {
        self.context.request_repaint_of(egui::ViewportId::ROOT);
    }
}

struct Session {
    events: Receiver<InputEvent>,
    statuses: Receiver<Status>,
    overflow: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Default)]
pub(crate) struct GlobalInput {
    session: Option<Session>,
    status: Status,
}

impl GlobalInput {
    pub(crate) fn status(&self) -> &Status {
        &self.status
    }

    pub(crate) fn start(&mut self, context: &egui::Context) {
        self.stop();
        if !platform::has_permission() {
            self.status = Status::PermissionRequired;
            return;
        }
        let (events_tx, events) = mpsc::sync_channel(QUEUE_CAPACITY);
        let (statuses_tx, statuses) = mpsc::channel();
        let overflow = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let sink = Sink {
            events: events_tx,
            statuses: statuses_tx,
            overflow: Arc::clone(&overflow),
            stopped: Arc::clone(&stopped),
            context: context.clone(),
        };
        match thread::Builder::new()
            .name("bongodsh-keyboard".to_owned())
            .spawn(move || {
                match platform::listen(&sink) {
                    Err(status) => {
                        sink.input(InputEvent::Reset);
                        sink.status(status);
                    }
                    Ok(()) if !sink.stopped.load(Ordering::Acquire) => {
                        sink.status(Status::Failed("Keyboard listener stopped".to_owned()));
                    }
                    Ok(()) => {}
                }
                // Also wakes the UI if the listener returned unexpectedly.
                sink.wake();
            }) {
            Ok(thread) => {
                self.session = Some(Session {
                    events,
                    statuses,
                    overflow,
                    stopped,
                    thread: Some(thread),
                });
                self.status = Status::Starting;
            }
            Err(error) => self.status = Status::Failed(error.to_string()),
        }
    }

    pub(crate) fn stop(&mut self) {
        self.session = None;
        self.status = Status::Off;
    }

    pub(crate) fn poll(&mut self) -> Vec<InputEvent> {
        let Some(session) = &self.session else {
            return Vec::new();
        };
        for status in session.statuses.try_iter() {
            self.status = status;
        }
        let mut events: Vec<_> = session.events.try_iter().take(QUEUE_CAPACITY).collect();
        if session.overflow.load(Ordering::Acquire) {
            for _ in session.events.try_iter() {}
            session.overflow.store(false, Ordering::Release);
            events.clear();
            events.push(InputEvent::Reset);
        }
        if matches!(self.status, Status::PermissionRequired | Status::Failed(_)) {
            self.session = None;
            return vec![InputEvent::Reset];
        }
        if session.thread.as_ref().is_some_and(JoinHandle::is_finished) {
            self.session = None;
            self.status = Status::Failed("Keyboard listener stopped".to_owned());
            return vec![InputEvent::Reset];
        }
        events
    }
}
