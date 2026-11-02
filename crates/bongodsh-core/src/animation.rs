//! Time-stepped hand motion, with a minimum visible duration for short taps.

use std::time::Duration;

use crate::{Feedback, Hand, Pose};

const TAP_SECONDS: f32 = 0.075;
const MAX_STEP_SECONDS: f32 = 1.0 / 120.0;

/// Animated hand displacement. Zero is idle and one is fully pressed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HandPositions {
    /// Left hand displacement, with a small overshoot during rebound.
    pub left: f32,
    /// Right hand displacement, with a small overshoot during rebound.
    pub right: f32,
}

#[derive(Default)]
struct Spring {
    position: f32,
    velocity: f32,
    tap_remaining: f32,
}

impl Spring {
    fn strike(&mut self) {
        self.tap_remaining = TAP_SECONDS;
        self.velocity = self.velocity.max(8.0);
    }

    fn step(&mut self, held: bool, seconds: f32) {
        let target = if held || self.tap_remaining > 0.0 {
            1.0
        } else {
            0.0
        };
        self.tap_remaining = (self.tap_remaining - seconds).max(0.0);
        let acceleration = 520.0 * (target - self.position) - 27.0 * self.velocity;
        self.velocity += acceleration * seconds;
        self.position += self.velocity * seconds;

        if (self.position - target).abs() < 0.001 && self.velocity.abs() < 0.01 {
            self.position = target;
            self.velocity = 0.0;
        }
    }

    fn is_moving(&self, held: bool) -> bool {
        let target = if held { 1.0 } else { 0.0 };
        self.tap_remaining > 0.0 || self.position != target || self.velocity != 0.0
    }
}

/// Two damped springs driven by input feedback, without an internal clock.
#[derive(Default)]
pub struct HandAnimation {
    left: Spring,
    right: Spring,
    target: Pose,
}

impl HandAnimation {
    /// Updates held positions and starts a visible impulse for a fresh strike.
    pub fn apply(&mut self, feedback: Feedback) {
        self.target = feedback.pose;
        match feedback.strike {
            Some(Hand::Left) => self.left.strike(),
            Some(Hand::Right) => self.right.strike(),
            None => {}
        }
    }

    /// Advances motion, returning whether another animation frame is needed.
    ///
    /// Long pauses are capped and divided into short steps to keep the spring
    /// stable after window dragging, breakpoints, or sleep.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        let mut remaining = elapsed.as_secs_f32().min(0.05);
        while remaining > 0.0 {
            let step = remaining.min(MAX_STEP_SECONDS);
            self.left.step(self.target.left_pressed, step);
            self.right.step(self.target.right_pressed, step);
            remaining = (remaining - step).max(0.0);
        }
        self.left.is_moving(self.target.left_pressed)
            || self.right.is_moving(self.target.right_pressed)
    }

    /// Returns the current positions for drawing.
    pub fn positions(&self) -> HandPositions {
        HandPositions {
            left: self.left.position,
            right: self.right.position,
        }
    }
}
