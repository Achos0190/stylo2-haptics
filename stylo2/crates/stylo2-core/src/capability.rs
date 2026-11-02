//! Capability reporting.
//!
//! A backend advertises exactly what it can do. The [`HapticManager`] gates
//! every command on these flags so the public API never offers a control the
//! active transport cannot actually perform.
//!
//! [`HapticManager`]: crate::manager::HapticManager

use crate::profile::VibrationProfile;

/// Which stylus events a backend can emit.
///
/// Everything defaults to `false`. A flag is only set `true` once a backend can
/// genuinely source that event. The three "feedback-layer" events
/// (`connection`, `vibration_switch`, `function_feedback`) correspond to the
/// reported vendor callback interface; `button`, `double_tap` and `battery` are
/// listed because the specification asks about them, but remain `false` until a
/// backend verifies them on hardware.
///
/// Note: ordinary pen input (position, pressure, tilt, hover) is **not** here.
/// That is delivered by Android's standard `MotionEvent` pipeline and this
/// library deliberately does not reimplement it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EventCapabilities {
    /// Can report stylus connect/disconnect transitions.
    pub connection: bool,
    /// Can report changes to the pen's vibration on/off switch.
    pub vibration_switch: bool,
    /// Can report changes to the "function feedback" state.
    pub function_feedback: bool,
    /// Can report a stylus button press. (Unverified; default `false`.)
    pub button: bool,
    /// Can report a stylus double-tap gesture. (Unverified; default `false`.)
    pub double_tap: bool,
    /// Can report battery-state changes. (Unverified; default `false`.)
    pub battery: bool,
}

/// What a backend can do.
///
/// Construct with [`Capabilities::none`] and set only the fields you can back
/// up, or use [`Capabilities::profile_switch_only`] for the common case of an
/// interface that offers profile selection plus start/stop and nothing more.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capabilities {
    /// Profiles the backend advertises. Empty means "no profile control".
    pub profiles: Vec<VibrationProfile>,
    /// Supports [`start_feedback`]/[`stop_feedback`].
    ///
    /// [`start_feedback`]: crate::manager::StylusHaptics::start_feedback
    /// [`stop_feedback`]: crate::manager::StylusHaptics::stop_feedback
    pub start_stop: bool,
    /// Supports a caller-chosen intensity. (Reported interface: no.)
    pub intensity: bool,
    /// Supports a caller-chosen duration. (Reported interface: no.)
    pub duration: bool,
    /// Supports a caller-chosen frequency. (Reported interface: no.)
    pub frequency: bool,
    /// Supports a caller-chosen waveform. (Reported interface: no.)
    pub waveform: bool,
    /// Supports uploading custom haptic patterns. (Reported interface: no.)
    pub custom_patterns: bool,
    /// Which events the backend can emit.
    pub events: EventCapabilities,
}

impl Capabilities {
    /// A backend that can do nothing. Useful as a starting point and as the
    /// honest default for an unconnected or unauthorised backend.
    pub fn none() -> Self {
        Capabilities::default()
    }

    /// The capability set of an interface that offers the four profiles plus
    /// start/stop and the three feedback-layer events — and nothing else.
    ///
    /// This matches the *reported* shape of the OnePlus accessory interface.
    /// It deliberately leaves intensity/duration/frequency/waveform/custom
    /// patterns `false`.
    pub fn profile_switch_only() -> Self {
        Capabilities {
            profiles: VibrationProfile::ALL.to_vec(),
            start_stop: true,
            events: EventCapabilities {
                connection: true,
                vibration_switch: true,
                function_feedback: true,
                ..EventCapabilities::default()
            },
            ..Capabilities::default()
        }
    }

    /// Whether the backend advertises the given profile.
    pub fn supports_profile(&self, profile: VibrationProfile) -> bool {
        self.profiles.contains(&profile)
    }
}
