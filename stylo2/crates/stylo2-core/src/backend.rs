//! The transport abstraction every haptic route implements.

use crate::capability::Capabilities;
use crate::error::StyloError;
use crate::event::EventCallback;
use crate::profile::VibrationProfile;
use crate::state::ConnectionState;

/// Identifies which concrete transport is behind a [`HapticBackend`].
///
/// Purely informational — the public API does not change with the backend.
///
/// [`HapticBackend`]: crate::backend::HapticBackend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendId {
    /// OnePlus accessory service (`com.oplus.ipemanager`) via binder.
    OplusIpe,
    /// Direct Bluetooth Low Energy to the pen peripheral.
    Ble,
    /// Android tablet vibrator motor — a clearly-labelled fallback that does
    /// **not** drive the pen's own motor.
    AndroidVibrator,
    /// In-memory test double.
    Mock,
}

/// A single haptic transport.
///
/// Implementations own all platform/transport specifics. They are expected to
/// be honest: `connection_state` and `capabilities` must reflect reality, and a
/// command must return the precise [`StyloError`] on failure rather than a fake
/// `Ok(())`. The [`HapticManager`] relies on this contract to sequence and gate
/// commands.
///
/// Methods take `&mut self`; the manager serialises access behind a lock, so
/// implementations need not be internally synchronised for the command path.
///
/// [`HapticManager`]: crate::manager::HapticManager
pub trait HapticBackend: Send {
    /// Which transport this is.
    fn id(&self) -> BackendId;

    /// The current connection state. Must be cheap and non-blocking.
    fn connection_state(&self) -> ConnectionState;

    /// What this backend can do *right now*. An unconnected or unauthorised
    /// backend should report [`Capabilities::none`].
    ///
    /// [`Capabilities::none`]: crate::capability::Capabilities::none
    fn capabilities(&self) -> Capabilities;

    /// Begin (or re-check) a connection. Returns `Ok(())` once the backend is
    /// usable; otherwise the precise error. Implementations should also update
    /// `connection_state` so observers see the transition.
    fn connect(&mut self) -> Result<(), StyloError>;

    /// Tear down the connection. Idempotent.
    fn disconnect(&mut self) -> Result<(), StyloError>;

    /// Select the active vibration profile on the device.
    fn set_profile(&mut self, profile: VibrationProfile) -> Result<(), StyloError>;

    /// Start feedback vibration using the currently selected profile.
    fn start_feedback(&mut self) -> Result<(), StyloError>;

    /// Stop feedback vibration.
    fn stop_feedback(&mut self) -> Result<(), StyloError>;

    /// Register (or clear, with `None`) the listener for asynchronous events.
    /// Backends that cannot source events may ignore this.
    fn set_event_callback(&mut self, callback: Option<EventCallback>);
}
