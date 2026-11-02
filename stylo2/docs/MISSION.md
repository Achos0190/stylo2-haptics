# Mission — OnePlus Stylo 2 full-stack interoperability investigation

The canonical brief for this project. Where it and the original task framing
differ, this governs. It is an **evidence-driven interoperability
investigation** on hardware the end user owns: enable native Stylo 2 haptics in
third-party apps on a **stock, non-rooted OnePlus Pad 3 (OxygenOS 16 /
Android 16)**, or establish with evidence exactly which authorization boundary
prevents it and what the least-restrictive legitimate alternative is.

## Non-negotiables (from the mission's operating principles)

- Ownership of the device is the starting point; investigate the real
  mechanisms (signatures, Binder authz, permission enforcement, BT session,
  protocol framing) directly.
- **No root, Magisk/KernelSU, bootloader unlock, or system/firmware changes.**
  Preserve stock OS, existing stylus input, and OnePlus Notes.
- Separate **technical feasibility** from **platform policy**, **risk**, and
  **legal**. Prefer a working, reproducible result over speculation.
- **Invent nothing** — no keys, packet formats, UUIDs, permissions, method
  signatures, or authorization behavior. Nothing is "works" until tested or
  evidenced. Don't misrepresent the app's identity or claim credentials not
  legitimately obtained.
- If an approach is unavailable under the constraints, pin the **specific
  reason and enforcement point**, then name the least-restrictive viable path.

## Deliverables → where they live

| # | Deliverable | Location | State |
| --- | --- | --- | --- |
| 1 | Architecture diagram | [`H-trust-boundaries.md`](H-trust-boundaries.md) | done |
| 2 | Trust-boundary report (mechanism + enforcement point) | [`H-trust-boundaries.md`](H-trust-boundaries.md) | done |
| 3 | Source & evidence ledger (verified/reported/hypothesis) | [`I-evidence-ledger.md`](I-evidence-ledger.md) | done |
| 4 | Packet-capture analysis (reproducible action↔traffic) | [`J-capture-plan.md`](J-capture-plan.md) | method + empty tables; fill on device |
| 5 | Feasibility matrix | [`B-feasibility.md`](B-feasibility.md) | done |
| 6 | Rust implementation plan | [`D-implementation-plan.md`](D-implementation-plan.md) + layout note below | done |
| 7 | Minimal Android test app | [`../../android/stylo2-poc`](../../android/stylo2-poc) (Route-A haptic PoC, 4-control screen) + [`../../android/stylus-ble-inspector`](../../android/stylus-ble-inspector) (BLE recon) | done (source; build + run on device) |
| 8 | Physical Pad 3 test report | [`E-prototype.md`](E-prototype.md) matrix + `J` tables | **pending hardware** |
| 9 | Unresolved questions + exact experiment for each | [`I-evidence-ledger.md`](I-evidence-ledger.md) §open questions | done |

On-device procedure: [`G-local-continuation.md`](G-local-continuation.md). Agent
handoff: [`../../PICKUP.md`](../../PICKUP.md). Legal framing:
[`LEGAL.md`](LEGAL.md).

## Experimental method

Nine controlled, repeatable captures (idle, the four profile selections,
feedback on, feedback off, disconnect/reconnect, initial session) with a fixed
recording schema — see [`J-capture-plan.md`](J-capture-plan.md). HCI snoop log
first; an external BLE sniffer only if the snoop is insufficient, and even then
link encryption may limit visibility.

## Architecture / API layout decisions (deliberate, evidence-driven)

The mission proposes a richer workspace (`stylo2-protocol`, `stylo2-transport`,
`stylo2-tests`) and a 7-state `ConnectionState`. Per the mission's own
principles — build the implementation only after establishing the protocol, and
prefer working code over speculative structure — the repo keeps its current,
host-verified layout now and grows into the richer shape **as evidence
arrives**, rather than shipping empty crates and unreachable states. The mapping
is explicit so nothing is lost:

| Mission crate | Current home | Grows into a separate crate when… |
| --- | --- | --- |
| `stylo2-core` (API, state machine, errors, profiles) | `stylo2-core` ✓ | — |
| `stylo2-transport` (transport-independent command interface) | `stylo2-core::backend::HapticBackend` (already the seam) | only if a second consumer needs it split out |
| `stylo2-protocol` (validated packet structs, encode/decode) | started as `stylo2-ble::recon`; **no device bytes yet** | a capture yields real framing to encode/decode |
| `stylo2-ble` | `stylo2-ble` ✓ | — |
| `stylo2-android` (+ Kotlin/JNI) | `stylo2-android` + `../../android/stylo2-bridge` ✓ | — |
| `stylo2-tests` | per-crate unit tests + inspector JVM tests | integration tests against a real transport exist |

**State machine.** The provisional 7-state lifecycle
(`Disconnected → Discovering → Connecting → Connected → Initialising → Ready`,
plus `Failed`) is the target once a real session/BLE flow is verified. The code
ships the **honest subset that something actually drives today** —
`Disconnected, Connecting, Connected, PermissionDenied, ServiceUnavailable` —
because adding `Discovering`/`Initialising`/`Ready`/`Failed` with no code path
to set them would be unreachable surface, which the "invent nothing / nothing
works until evidenced" principles argue against. The target lifecycle is drawn
in [`H-trust-boundaries.md`](H-trust-boundaries.md); the enum is widened in the
same commit that first drives the new states (e.g. when the BLE session flow
lands). `PermissionDenied`/`ServiceUnavailable` are kept as first-class states
(they carry honest "why it won't work") and map onto the mission's `Failed`.

## Acceptance criteria (tracked)

Success = the actual cause of the third-party limitation is identified and
evidenced; the chosen method is evidence-backed; each verified profile selects;
feedback on/off is reliable; reconnection/lifecycle are handled; normal input
and Notes keep working; no root/system changes; no undocumented behavior
presented as fact; any remaining restriction is explicitly documented. If
unmet, ship the strongest reproducible finding and name exactly which criterion
is unsatisfied and why. Progress against these lives in the evidence ledger and
the `E` test matrix.
