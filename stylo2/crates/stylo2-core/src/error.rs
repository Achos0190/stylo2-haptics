//! Error type for every fallible haptic operation.

use std::fmt;

/// All the ways a haptic operation can fail.
///
/// The variants are deliberately specific so the UI (and the FFI/Kotlin layer)
/// can distinguish *"the device will never allow this"* from *"not connected
/// yet"* from *"the hardware does not expose this control"*. That distinction
/// is the whole point of the project's honesty requirement: an unauthorised or
/// unsupported call must surface as a precise error, never as a fake success.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StyloError {
    /// No usable connection to a backend. Call `connect()` first, or the
    /// stylus/service is not currently available.
    NotConnected,

    /// The OS refused access because the app lacks a required permission
    /// (e.g. the restricted `com.oplus.permission.safe.IOT` on the binder
    /// route). This is terminal for an ordinary app on stock firmware.
    PermissionDenied,

    /// The target service/peripheral could not be found or bound.
    ServiceUnavailable,

    /// The transport connected, but the vendor layer rejected this app's
    /// identity (vendor SDK registration / app authorisation required).
    /// Distinct from [`Self::PermissionDenied`]: the OS allowed the bind, the
    /// *vendor* declined the caller.
    Unauthorized,

    /// The feature exists but the user has turned the pen's vibration switch
    /// off. Reported by the backend; surfaced so the app can prompt the user
    /// rather than silently doing nothing.
    VibrationDisabled,

    /// The requested control is not exposed by the active backend's interface
    /// (e.g. asking for intensity control on an interface that only supports
    /// profile + start/stop). Carries a short human-readable reason.
    Unsupported(String),

    /// A profile was requested that the active backend does not advertise, or
    /// a command was issued in an invalid order (e.g. start with no profile
    /// selected). Carries a short human-readable reason.
    InvalidRequest(String),

    /// The underlying transport failed (binder `RemoteException`, GATT write
    /// failure, disconnect mid-call, …). Carries a short description.
    Transport(String),

    /// A backend-internal error that does not fit the categories above.
    Backend(String),
}

impl fmt::Display for StyloError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StyloError::NotConnected => f.write_str("not connected to a haptic backend"),
            StyloError::PermissionDenied => {
                f.write_str("permission denied by the operating system")
            }
            StyloError::ServiceUnavailable => {
                f.write_str("haptic service or peripheral unavailable")
            }
            StyloError::Unauthorized => {
                f.write_str("vendor layer refused this application's identity")
            }
            StyloError::VibrationDisabled => {
                f.write_str("stylus vibration is switched off by the user")
            }
            StyloError::Unsupported(why) => write!(f, "unsupported control: {why}"),
            StyloError::InvalidRequest(why) => write!(f, "invalid request: {why}"),
            StyloError::Transport(why) => write!(f, "transport failure: {why}"),
            StyloError::Backend(why) => write!(f, "backend error: {why}"),
        }
    }
}

impl std::error::Error for StyloError {}
