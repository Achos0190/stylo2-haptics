# Deliverable B — Feasibility assessment

Scope constraint for every row: **stock OxygenOS, non-rooted, the app signed
with its own independently generated certificate, no privileged
(`/system/priv-app`) install, no firmware modification.**

## The four routes

| Route | Feasibility (stock, non-root, own cert) | Dependencies | Key limitations |
| --- | --- | --- | --- |
| **Official OnePlus SDK** | **Unknown — the only *legitimate* path to privileged-equivalent access, but existence/openness unconfirmed** | A OnePlus/OPPO developer program that authorises this app's package + signing certificate for the accessory service | No public SDK/docs found (A §5). May require business/NDA terms; may not cover Pad 3/Stylo 2; cannot be assumed to exist |
| **Binder — IpeManager** | **Not available to an ordinary app.** The AIDL calls themselves are fine; the *bind* is gated | `com.oplus.permission.safe.IOT` (restricted, ≥privileged — A §3), and possibly a vendor signature check | An ordinary app gets `PermissionDenied`. The only ways to hold the permission are (a) privileged/system install = root → **out of scope**, or (b) vendor authorisation = the "Official SDK" row |
| **Direct BLE** | **Unknown — unverified, possibly impossible** | A verified writable haptic GATT characteristic + command encoding + proof a 2nd connection doesn't break the system link | Protocol entirely unverified (A §4). May be proprietary/encrypted or exclusively owned by the system connection. Standard BT permissions are grantable, so *this* part is not the blocker |
| **Android vibrator fallback** | **Available in principle** | `android.os.Vibrator`/`VibratorManager` (+ BT permissions if gating on pen presence) | **Tablet motor, not the pen.** No profiles; not real stylus haptics. A clearly-labelled degraded mode only |

## Honest bottom line

For a stock, non-rooted Pad 3 with the app's own certificate, **there is no
route to the pen's own haptic motor that is verified to work today.** This lands
squarely in the specification's category *"access requiring functionality
unavailable on stock, non-rooted firmware"* for the binder route, and
*"to establish"* for the SDK and BLE routes.

That is a finding, not a dead end. Two routes remain genuinely open and must be
settled **on the device / with the vendor**, not from a desk:

1. **Official SDK / developer authorisation** — the only lawful way to obtain
   binder access without root. Action: approach OnePlus/OPPO developer
   relations; establish whether package + certificate authorisation is offered
   for the accessory service, and on what terms. If yes, the binder backend
   already in this workspace is the implementation.
2. **Direct BLE** — the only root-free route that needs no vendor cooperation,
   *if* the protocol turns out to be reachable. Action: run the on-device BLE
   reconnaissance in `C-protocol.md`. If a writable haptic characteristic and
   encoding are found and a second connection is non-disruptive, the BLE backend
   is the implementation.

The tablet-vibrator fallback is implemented and labelled, so the app is never
dead — but it is explicitly *not* the goal.

## Preferred implementation route

**Architect for all three; commit to whichever the device/vendor proves out,
behind one API.** Concretely, the recommended order of *investigation* is:

1. **BLE reconnaissance first** (cheap, needs only the tablet + user consent, no
   vendor dependency). It can be done immediately and either unlocks the fully
   independent route or rules it out with evidence.
2. **Official-SDK enquiry in parallel** (the only legitimate binder route; gated
   on an external party, so start the clock early).
3. **Tablet fallback** ships meanwhile as an honest degraded mode.

The library's [`stylo2_android::preferred_backend`] already encodes this
precedence (pen-driving routes first, fallback last) so that attaching a proven
transport is the only change needed to "light up" a route.

This avoids the two failure modes the spec warns about: it does **not** assume
BLE is possible, and it does **not** declare the binder route impossible in
principle — it pins the binder route's blocker precisely (a restricted
permission + out-of-scope privileged install) and names the one legitimate way
around it (vendor authorisation).

[`stylo2_android::preferred_backend`]: ../crates/stylo2-android/src/lib.rs
