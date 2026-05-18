use std::collections::VecDeque;

/// Centralized notification queue for transient UI error/success messages.
/// Each message has an expiry timestamp.
#[derive(Default)]
pub struct ErrorState {
    messages: VecDeque<(String, f64)>,
}

impl ErrorState {
    /// Push a new message. It will be displayed for 3 seconds.
    pub fn push(&mut self, msg: impl Into<String>) {
        self.messages.push_back((msg.into(), 0.0));
    }

    /// Push a message with a specific expiry timestamp.
    /// `time` should be the current wall clock time (from `ctx.input(|i| i.time)`).
    /// The message will show until `time + 3.0`.
    pub fn push_with_expiry(&mut self, msg: impl Into<String>, time: f64) {
        self.messages.push_back((msg.into(), time + 3.0));
    }

    /// Clear all expired messages. Call this each frame with the current time.
    pub fn update(&mut self, time: f64) {
        self.messages.retain(|(_, expiry)| *expiry == 0.0 || *expiry > time);
    }

    /// Iterate over active messages.
    pub fn active(&self) -> impl Iterator<Item = &str> {
        self.messages.iter().map(|(msg, _)| msg.as_str())
    }

    /// Returns true if there are any active messages.
    #[allow(dead_code)]
    pub fn has_active(&self) -> bool {
        !self.messages.is_empty()
    }
}