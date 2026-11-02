//! Vibration profiles.

/// A haptic "feel" profile for the stylus.
///
/// These are the four profiles reported by the stock Notes integration and the
/// existing community implementation. They are an *abstract* set here on
/// purpose:
///
/// **This crate does not assign any integer or wire value to a profile.** The
/// community implementation happens to pass `enum.ordinal()` to the vendor
/// `setVibrationType(int)` call, but the project specification is explicit that
/// the real integer mapping must be verified against the actual service and
/// must not be assumed from declaration order. The mapping therefore lives in
/// each backend (see `stylo2-oplus`) where it can be corrected in exactly one
/// place once confirmed on hardware, and is labelled *reported / unverified*
/// until then.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VibrationProfile {
    /// Pencil feel.
    Pencil,
    /// Eraser feel.
    Eraser,
    /// Ballpoint-pen feel.
    Ballpoint,
    /// Fountain/marker "pen" feel.
    Pen,
}

impl VibrationProfile {
    /// Every profile this crate knows about, in a stable order for UI lists.
    ///
    /// Note: this order is for display only and carries **no** protocol
    /// meaning (see the type-level docs).
    pub const ALL: [VibrationProfile; 4] = [
        VibrationProfile::Pencil,
        VibrationProfile::Eraser,
        VibrationProfile::Ballpoint,
        VibrationProfile::Pen,
    ];

    /// A stable, lowercase identifier suitable for logs and the FFI boundary.
    pub fn as_str(self) -> &'static str {
        match self {
            VibrationProfile::Pencil => "pencil",
            VibrationProfile::Eraser => "eraser",
            VibrationProfile::Ballpoint => "ballpoint",
            VibrationProfile::Pen => "pen",
        }
    }

    /// Parse a profile from its identifier, case-insensitively.
    ///
    /// Accepts the canonical names plus the `ballpen` alias used by the vendor
    /// enum. Returns `None` for anything unknown rather than guessing.
    pub fn from_str_ci(s: &str) -> Option<VibrationProfile> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pencil" => Some(VibrationProfile::Pencil),
            "eraser" => Some(VibrationProfile::Eraser),
            "ballpoint" | "ballpen" => Some(VibrationProfile::Ballpoint),
            "pen" => Some(VibrationProfile::Pen),
            _ => None,
        }
    }
}
