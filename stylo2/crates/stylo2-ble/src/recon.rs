//! Turning on-device GATT discovery into candidate control characteristics.
//!
//! The Stylo2 BLE Inspector (`android/stylus-ble-inspector`) exports a
//! discovered GATT table. Feed that table in here to rank which characteristics
//! are plausible haptic-control attributes — the first step toward drafting a
//! (still human-verified) [`Stylo2GattProfile`](crate::Stylo2GattProfile).
//!
//! This is a heuristic ranker, not a detector. It never claims a characteristic
//! *is* the control; it orders candidates so a human knows where to probe.

/// A characteristic as reported by on-device discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredCharacteristic {
    /// Owning service UUID (string form).
    pub service: String,
    /// Characteristic UUID (string form).
    pub uuid: String,
    /// Has the WRITE (with response) property.
    pub write: bool,
    /// Has the WRITE_NO_RESPONSE property.
    pub write_no_response: bool,
    /// Has the NOTIFY property.
    pub notify: bool,
    /// Has the INDICATE property.
    pub indicate: bool,
}

impl DiscoveredCharacteristic {
    fn writable(&self) -> bool {
        self.write || self.write_no_response
    }
}

/// A ranked candidate control characteristic with the reasoning behind its score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlCandidate {
    /// Owning service UUID.
    pub service: String,
    /// Candidate characteristic UUID.
    pub uuid: String,
    /// Heuristic score (higher = more likely a vendor control attribute).
    pub score: u8,
    /// Human-readable reasons contributing to the score.
    pub reasons: Vec<String>,
}

/// True if `uuid` is a 16-/32-bit Bluetooth SIG UUID expressed against the
/// Bluetooth base UUID (`xxxxxxxx-0000-1000-8000-00805f9b34fb`). Such
/// characteristics are standardised and unlikely to be a proprietary control.
pub fn is_standard_sig_uuid(uuid: &str) -> bool {
    let u = uuid.trim().to_ascii_lowercase();
    u.len() == 36 && u.ends_with("-0000-1000-8000-00805f9b34fb")
}

/// Rank the writable characteristics in a discovered GATT table as candidate
/// haptic-control attributes, best first.
///
/// Heuristics (all surfaced in `reasons`):
/// * writability is required — non-writable characteristics are excluded;
/// * a vendor (128-bit, non-SIG) UUID is up-ranked over a standard SIG one;
/// * a characteristic whose service also exposes a notify/indicate attribute
///   (a plausible response channel) is up-ranked.
pub fn candidate_control_characteristics(
    chars: &[DiscoveredCharacteristic],
) -> Vec<ControlCandidate> {
    let mut services_with_response: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for c in chars {
        if c.notify || c.indicate {
            services_with_response.insert(c.service.to_ascii_lowercase());
        }
    }

    let mut out: Vec<ControlCandidate> = Vec::new();
    for c in chars {
        if !c.writable() {
            continue;
        }
        let mut score: u8 = 50;
        let mut reasons: Vec<String> = Vec::new();

        reasons.push(
            if c.write {
                "writable with response"
            } else {
                "writable without response"
            }
            .to_string(),
        );

        if is_standard_sig_uuid(&c.uuid) {
            score = score.saturating_sub(30);
            reasons.push("standard SIG UUID — unlikely to be a vendor control".to_string());
        } else {
            score = score.saturating_add(20);
            reasons.push("vendor (non-SIG) UUID".to_string());
        }

        if services_with_response.contains(&c.service.to_ascii_lowercase()) {
            score = score.saturating_add(20);
            reasons.push("service also exposes a notify/indicate response channel".to_string());
        }

        out.push(ControlCandidate {
            service: c.service.clone(),
            uuid: c.uuid.clone(),
            score,
            reasons,
        });
    }

    // Highest score first; stable on ties so input order is preserved.
    out.sort_by_key(|c| std::cmp::Reverse(c.score));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(service: &str, uuid: &str, write: bool, wnr: bool, notify: bool) -> DiscoveredCharacteristic {
        DiscoveredCharacteristic {
            service: service.to_string(),
            uuid: uuid.to_string(),
            write,
            write_no_response: wnr,
            notify,
            indicate: false,
        }
    }

    const SIG_BATTERY: &str = "00002a19-0000-1000-8000-00805f9b34fb";
    const VENDOR_CTRL: &str = "7f7e0001-1a2b-3c4d-5e6f-001122334455";
    const VENDOR_NOTIFY: &str = "7f7e0002-1a2b-3c4d-5e6f-001122334455";
    const VENDOR_SVC: &str = "7f7e00f0-1a2b-3c4d-5e6f-001122334455";

    #[test]
    fn recognises_sig_uuids() {
        assert!(is_standard_sig_uuid(SIG_BATTERY));
        assert!(is_standard_sig_uuid("0000FFE1-0000-1000-8000-00805F9B34FB")); // case-insensitive
        assert!(!is_standard_sig_uuid(VENDOR_CTRL));
    }

    #[test]
    fn excludes_non_writable() {
        let chars = vec![ch(VENDOR_SVC, VENDOR_NOTIFY, false, false, true)];
        assert!(candidate_control_characteristics(&chars).is_empty());
    }

    #[test]
    fn vendor_writable_with_response_channel_ranks_first() {
        let chars = vec![
            // standard writable (e.g. some SIG control) — low rank
            ch("00001801-0000-1000-8000-00805f9b34fb", SIG_BATTERY, true, false, false),
            // vendor writable in a service that also notifies — high rank
            ch(VENDOR_SVC, VENDOR_CTRL, true, false, false),
            // the notify partner in the same service
            ch(VENDOR_SVC, VENDOR_NOTIFY, false, false, true),
        ];
        let ranked = candidate_control_characteristics(&chars);
        assert_eq!(ranked.len(), 2, "only writable chars are candidates");
        assert_eq!(ranked[0].uuid, VENDOR_CTRL);
        assert!(ranked[0].score > ranked[1].score);
        assert!(ranked[0]
            .reasons
            .iter()
            .any(|r| r.contains("response channel")));
    }
}
