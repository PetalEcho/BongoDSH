use std::time::{Duration, Instant};

use bongodsh_core::{Feedback, Hand, InputEvent, TypingState, animation::HandAnimation};

const DEMO_INTERVALS_MS: [u64; 8] = [240, 240, 150, 150, 95, 95, 150, 400];

#[derive(Default, PartialEq)]
enum InputSource {
    #[default]
    Window,
    Preview,
}

struct Demo {
    index: usize,
    next_at: Instant,
}

#[derive(Default)]
pub(crate) struct Interaction {
    typing: TypingState,
    pub(crate) animation: HandAnimation,
    pub(crate) paused: bool,
    source: InputSource,
    next_hand: Hand,
    demo: Option<Demo>,
}

impl Interaction {
    pub(crate) fn reset(&mut self) {
        self.typing.apply(InputEvent::Reset);
        self.animation = HandAnimation::default();
        self.demo = None;
        self.source = InputSource::Window;
        self.next_hand = Hand::Left;
    }

    pub(crate) fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.reset();
    }

    pub(crate) fn lose_focus(&mut self) {
        if self.source == InputSource::Window {
            self.reset();
        }
    }

    pub(crate) fn input(&mut self, event: InputEvent) {
        if event == InputEvent::Reset {
            self.reset();
            return;
        }
        if self.paused {
            return;
        }
        if self.source != InputSource::Window {
            // A late release or OS repeat must not interrupt a manual preview.
            if !matches!(
                event,
                InputEvent::Press { repeat: false, .. } | InputEvent::Reset
            ) {
                return;
            }
            self.reset();
        }
        self.animation.apply(self.typing.apply(event));
    }

    pub(crate) fn tap(&mut self, hand: Option<Hand>) {
        if self.paused {
            return;
        }
        if self.source != InputSource::Preview {
            self.reset();
            self.source = InputSource::Preview;
        }
        self.demo = None;
        self.strike(hand.unwrap_or(self.next_hand));
    }

    fn strike(&mut self, hand: Hand) {
        self.animation.apply(Feedback {
            strike: Some(hand),
            ..Feedback::default()
        });
        self.next_hand = match hand {
            Hand::Left => Hand::Right,
            Hand::Right => Hand::Left,
        };
    }

    pub(crate) fn start_demo(&mut self, now: Instant) {
        if self.paused {
            return;
        }
        self.reset();
        self.source = InputSource::Preview;
        self.demo = Some(Demo {
            index: 0,
            next_at: now,
        });
    }

    pub(crate) fn is_demo_playing(&self) -> bool {
        self.demo.is_some()
    }

    pub(crate) fn is_previewing(&self) -> bool {
        self.source == InputSource::Preview
    }

    pub(crate) fn preview_finished(&self) -> bool {
        self.is_previewing() && self.demo.is_none() && !self.animation.is_moving()
    }

    /// Advance old motion before processing any input for the current frame.
    pub(crate) fn advance(&mut self, elapsed: Duration, now: Instant) {
        self.animation.advance(elapsed);
        let Some(demo) = &mut self.demo else { return };
        if now < demo.next_at {
            return;
        }
        demo.next_at = now + Duration::from_millis(DEMO_INTERVALS_MS[demo.index]);
        demo.index += 1;
        if demo.index == DEMO_INTERVALS_MS.len() {
            self.demo = None;
        }
        self.strike(self.next_hand);
    }

    pub(crate) fn next_repaint(&self, now: Instant) -> Option<Duration> {
        if self.animation.is_moving() {
            Some(Duration::from_millis(16))
        } else {
            self.demo
                .as_ref()
                .map(|demo| demo.next_at.saturating_duration_since(now))
        }
    }
}
