//! An in-memory [`HapticBackend`] for testing integrations without a device.
//!
//! Available to this crate's own tests automatically, and to downstream crates
//! via the `mock` feature. The mock is deliberately "dumb": it records the
//! commands it receives and returns success unless a failure is injected. All
//! the interesting *policy* (guarding, gating, de-duplication) lives in
//! [`HapticManager`], so testing against this mock tests that policy.
//!
//! [`HapticManager`]: crate::manager::HapticManager

use std::sync::{Arc, Mutex};

use crate::backend::{BackendId, HapticBackend};
use crate::capability::Capabilities;
use crate::error::StyloError;
use crate::event::{EventCallback, StylusEvent};
use crate::profile::VibrationProfile;
use crate::state::ConnectionState;

/// A command the mock received, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockCall {
    /// `connect()` was called.
    Connect,
    /// `disconnect()` was called.
    Disconnect,
    /// `set_profile(_)` was called.
    SetProfile(VibrationProfile),
    /// `start_feedback()` was called.
    StartFeedback,
    /// `stop_feedback()` was called.
    StopFeedback,
}

/// Shared, interior-mutable handle to a [`MockBackend`]'s state.
///
/// Clone one of these *before* boxing the backend into a [`HapticManager`] so a
/// test can still drive and inspect the backend afterwards.
///
/// [`HapticManager`]: crate::manager::HapticManager
#[derive(Clone)]
pub struct MockHandle {
    state: Arc<Mutex<ConnectionState>>,
    capabilities: Arc<Mutex<Capabilities>>,
    fail_next: Arc<Mutex<Option<StyloError>>>,
    calls: Arc<Mutex<Vec<MockCall>>>,
    callback: Arc<Mutex<Option<EventCallback>>>,
}

impl MockHandle {
    /// Set the state the backend will report.
    pub fn set_state(&self, state: ConnectionState) {
        *self.state.lock().unwrap() = state;
    }

    /// Set the capabilities the backend will report.
    pub fn set_capabilities(&self, caps: Capabilities) {
        *self.capabilities.lock().unwrap() = caps;
    }

    /// Make the next single command fail with this error.
    pub fn fail_next(&self, err: StyloError) {
        *self.fail_next.lock().unwrap() = Some(err);
    }

    /// The commands received so far, in order.
    pub fn calls(&self) -> Vec<MockCall> {
        self.calls.lock().unwrap().clone()
    }

    /// How many times a matching command was received.
    pub fn count(&self, call: &MockCall) -> usize {
        self.calls.lock().unwrap().iter().filter(|c| *c == call).count()
    }

    /// Fire an event to the registered listener, if any.
    pub fn emit(&self, event: StylusEvent) {
        let cb = self.callback.lock().unwrap().clone();
        if let Some(cb) = cb {
            cb(event);
        }
    }
}

/// In-memory test backend. Construct with [`MockBackend::new`].
pub struct MockBackend {
    handle: MockHandle,
}

impl MockBackend {
    /// A mock that starts `Disconnected` and reports the "profile + start/stop"
    /// capability set.
    pub fn new() -> Self {
        MockBackend {
            handle: MockHandle {
                state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
                capabilities: Arc::new(Mutex::new(Capabilities::profile_switch_only())),
                fail_next: Arc::new(Mutex::new(None)),
                calls: Arc::new(Mutex::new(Vec::new())),
                callback: Arc::new(Mutex::new(None)),
            },
        }
    }

    /// A handle for driving and inspecting this backend after it is boxed.
    pub fn handle(&self) -> MockHandle {
        self.handle.clone()
    }

    fn record(&self, call: MockCall) {
        self.handle.calls.lock().unwrap().push(call);
    }

    fn take_injected_failure(&self) -> Option<StyloError> {
        self.handle.fail_next.lock().unwrap().take()
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl HapticBackend for MockBackend {
    fn id(&self) -> BackendId {
        BackendId::Mock
    }

    fn connection_state(&self) -> ConnectionState {
        *self.handle.state.lock().unwrap()
    }

    fn capabilities(&self) -> Capabilities {
        self.handle.capabilities.lock().unwrap().clone()
    }

    fn connect(&mut self) -> Result<(), StyloError> {
        self.record(MockCall::Connect);
        if let Some(err) = self.take_injected_failure() {
            return Err(err);
        }
        let mut state = self.handle.state.lock().unwrap();
        match *state {
            ConnectionState::Connected => Ok(()),
            ConnectionState::PermissionDenied => Err(StyloError::PermissionDenied),
            ConnectionState::ServiceUnavailable => Err(StyloError::ServiceUnavailable),
            ConnectionState::Disconnected | ConnectionState::Connecting => {
                *state = ConnectionState::Connected;
                Ok(())
            }
        }
    }

    fn disconnect(&mut self) -> Result<(), StyloError> {
        self.record(MockCall::Disconnect);
        *self.handle.state.lock().unwrap() = ConnectionState::Disconnected;
        Ok(())
    }

    fn set_profile(&mut self, profile: VibrationProfile) -> Result<(), StyloError> {
        self.record(MockCall::SetProfile(profile));
        match self.take_injected_failure() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn start_feedback(&mut self) -> Result<(), StyloError> {
        self.record(MockCall::StartFeedback);
        match self.take_injected_failure() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn stop_feedback(&mut self) -> Result<(), StyloError> {
        self.record(MockCall::StopFeedback);
        match self.take_injected_failure() {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn set_event_callback(&mut self, callback: Option<EventCallback>) {
        *self.handle.callback.lock().unwrap() = callback;
    }
}
