//! Connection state.

/// Observable connection state of a haptic backend.
///
/// Mirrors the five states in the project specification. `PermissionDenied` and
/// `ServiceUnavailable` are first-class states (not just errors) because the UI
/// needs to render a stable "this will not work, and here is why" status, not
/// merely react to a one-off failed call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected, and not trying to.
    Disconnected,
    /// A connection attempt is in progress.
    Connecting,
    /// Connected and ready to accept commands.
    Connected,
    /// The OS denied the permission required to connect. Terminal for an
    /// ordinary app on stock firmware.
    PermissionDenied,
    /// The service or peripheral is not available on this device.
    ServiceUnavailable,
}

impl ConnectionState {
    /// `true` only when commands may be issued.
    pub fn is_usable(self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// `true` when the state is terminal — retrying `connect()` will not help
    /// without a change outside the app (granting a permission, pairing the
    /// pen, installing the vendor service).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ConnectionState::PermissionDenied | ConnectionState::ServiceUnavailable
        )
    }
}
