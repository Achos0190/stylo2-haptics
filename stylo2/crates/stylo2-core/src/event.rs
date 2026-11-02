//! Stylus events and the listener callback type.

use std::sync::Arc;

/// An asynchronous event emitted by a backend.
///
/// Only events the project can plausibly source from the *reported* vendor
/// callback interface are represented. Button / double-tap / battery events are
/// intentionally absent until a backend verifies them on hardware (see
/// [`EventCapabilities`]); adding a variant here would imply a capability the
/// library has not demonstrated.
///
/// Ordinary pen input (motion, pressure, tilt, hover, buttons as input) is not
/// modelled here — that belongs to Android's `MotionEvent` pipeline.
///
/// [`EventCapabilities`]: crate::capability::EventCapabilities
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StylusEvent {
    /// The stylus became connected to the active backend.
    Connected,
    /// The stylus disconnected from the active backend.
    Disconnected,
    /// The pen's vibration on/off switch changed. `true` = vibration enabled.
    VibrationSwitchChanged(bool),
    /// The "function feedback" state changed. `true` = enabled.
    FunctionFeedbackChanged(bool),
}

/// A thread-safe listener invoked when a backend emits a [`StylusEvent`].
///
/// `Arc<dyn Fn ...>` so it can be shared with a backend worker thread and with
/// the FFI layer, and cloned cheaply.
pub type EventCallback = Arc<dyn Fn(StylusEvent) + Send + Sync>;
