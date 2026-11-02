//! The public facade and the command-sequencing logic.

use std::sync::Mutex;

use crate::backend::HapticBackend;
use crate::capability::Capabilities;
use crate::error::StyloError;
use crate::event::EventCallback;
use crate::profile::VibrationProfile;
use crate::state::ConnectionState;

/// The stable public API a third-party app programs against.
///
/// Takes `&self` throughout so a single handle can be shared across threads and
/// held for a process's lifetime behind the FFI boundary.
pub trait StylusHaptics {
    /// Current connection state of the active backend.
    fn connection_state(&self) -> ConnectionState;

    /// What the active backend can do right now.
    fn capabilities(&self) -> Capabilities;

    /// Select the vibration profile. No-op (returns `Ok`) if the profile is
    /// already selected.
    fn set_profile(&self, profile: VibrationProfile) -> Result<(), StyloError>;

    /// Start feedback vibration. No-op (returns `Ok`) if already active.
    fn start_feedback(&self) -> Result<(), StyloError>;

    /// Stop feedback vibration. No-op (returns `Ok`) if already stopped.
    fn stop_feedback(&self) -> Result<(), StyloError>;
}

/// Wraps any [`HapticBackend`] and enforces correct, de-duplicated, honestly
/// reported command sequencing.
///
/// Responsibilities (all verified by this crate's tests):
///
/// * **Guarding** — every command first checks the backend's connection state
///   and maps a non-`Connected` state to the precise error
///   ([`PermissionDenied`]/[`ServiceUnavailable`]/[`NotConnected`]).
/// * **Capability gating** — a command for a control the backend does not
///   advertise returns [`Unsupported`] and is never sent downward.
/// * **De-duplication** — re-selecting the current profile, double-starting, or
///   stopping when already stopped are no-ops, so duplicate/conflicting
///   commands never reach the device.
/// * **Honest state** — the cached `active`/`profile` flags are only advanced
///   when the backend call actually succeeds. A failed call leaves state
///   unchanged and returns the error.
///
/// [`PermissionDenied`]: StyloError::PermissionDenied
/// [`ServiceUnavailable`]: StyloError::ServiceUnavailable
/// [`NotConnected`]: StyloError::NotConnected
/// [`Unsupported`]: StyloError::Unsupported
pub struct HapticManager {
    inner: Mutex<Inner>,
}

struct Inner {
    backend: Box<dyn HapticBackend>,
    current_profile: Option<VibrationProfile>,
    feedback_active: bool,
}

impl HapticManager {
    /// Build a manager over a concrete backend.
    pub fn new(backend: Box<dyn HapticBackend>) -> Self {
        HapticManager {
            inner: Mutex::new(Inner {
                backend,
                current_profile: None,
                feedback_active: false,
            }),
        }
    }

    /// Begin (or re-check) the backend connection.
    pub fn connect(&self) -> Result<(), StyloError> {
        let mut inner = self.lock();
        inner.backend.connect()
    }

    /// Tear down the backend connection and reset cached command state.
    pub fn disconnect(&self) -> Result<(), StyloError> {
        let mut inner = self.lock();
        // Best-effort: if feedback is running, ask the backend to stop before
        // dropping the link, but do not let a stop failure mask disconnect.
        if inner.feedback_active {
            let _ = inner.backend.stop_feedback();
            inner.feedback_active = false;
        }
        inner.current_profile = None;
        inner.backend.disconnect()
    }

    /// Whether feedback is currently believed to be running.
    pub fn is_feedback_active(&self) -> bool {
        self.lock().feedback_active
    }

    /// The currently selected profile, if any.
    pub fn current_profile(&self) -> Option<VibrationProfile> {
        self.lock().current_profile
    }

    /// Register a listener for asynchronous stylus events (or clear it with
    /// `None`).
    pub fn set_event_listener(&self, callback: Option<EventCallback>) {
        self.lock().backend.set_event_callback(callback);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // A poisoned lock means a previous command panicked mid-call. Recover
        // the guard rather than propagating the panic; the next command will
        // re-check backend state anyway.
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// Map the backend's connection state to a precise error, or `Ok` if usable.
fn require_connected(state: ConnectionState) -> Result<(), StyloError> {
    match state {
        ConnectionState::Connected => Ok(()),
        ConnectionState::PermissionDenied => Err(StyloError::PermissionDenied),
        ConnectionState::ServiceUnavailable => Err(StyloError::ServiceUnavailable),
        ConnectionState::Disconnected | ConnectionState::Connecting => {
            Err(StyloError::NotConnected)
        }
    }
}

impl StylusHaptics for HapticManager {
    fn connection_state(&self) -> ConnectionState {
        self.lock().backend.connection_state()
    }

    fn capabilities(&self) -> Capabilities {
        self.lock().backend.capabilities()
    }

    fn set_profile(&self, profile: VibrationProfile) -> Result<(), StyloError> {
        let mut inner = self.lock();
        require_connected(inner.backend.connection_state())?;

        if !inner.backend.capabilities().supports_profile(profile) {
            return Err(StyloError::Unsupported(format!(
                "backend does not advertise the {} profile",
                profile.as_str()
            )));
        }

        // De-duplicate: re-selecting the active profile is a no-op.
        if inner.current_profile == Some(profile) {
            return Ok(());
        }

        inner.backend.set_profile(profile)?;
        inner.current_profile = Some(profile);
        Ok(())
    }

    fn start_feedback(&self) -> Result<(), StyloError> {
        let mut inner = self.lock();
        require_connected(inner.backend.connection_state())?;

        if !inner.backend.capabilities().start_stop {
            return Err(StyloError::Unsupported(
                "backend does not support start/stop feedback".to_string(),
            ));
        }

        // Refuse an ambiguous start: for a backend that exposes profiles, one
        // must be selected first, so the device is never asked to vibrate with
        // an undefined feel. A profileless backend (e.g. the tablet-vibrator
        // fallback) is exempt.
        let has_profiles = !inner.backend.capabilities().profiles.is_empty();
        if has_profiles && inner.current_profile.is_none() {
            return Err(StyloError::InvalidRequest(
                "select a vibration profile before starting feedback".to_string(),
            ));
        }

        // De-duplicate: do not double-start.
        if inner.feedback_active {
            return Ok(());
        }

        inner.backend.start_feedback()?;
        inner.feedback_active = true;
        Ok(())
    }

    fn stop_feedback(&self) -> Result<(), StyloError> {
        let mut inner = self.lock();

        // De-duplicate: stopping when already stopped is a success no-op, and
        // needs no live connection.
        if !inner.feedback_active {
            return Ok(());
        }

        require_connected(inner.backend.connection_state())?;
        inner.backend.stop_feedback()?;
        inner.feedback_active = false;
        Ok(())
    }
}
