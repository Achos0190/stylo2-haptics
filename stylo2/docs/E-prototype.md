# Deliverable E — Working prototype

## What exists now

- A **compiled, tested** portable core + backends + C ABI (host-verified: 22
  tests, demo runs). `cargo run -p stylo2-core --example demo --features mock`
  shows the full command lifecycle and honest error reporting without a device.
- The Kotlin bridge source (`android/stylo2-bridge`) and the existing app, which
  together form the on-device prototype once built with the Android SDK/NDK.
- The **Stylo2 BLE Inspector** (`android/stylus-ble-inspector`), a standalone
  non-root testing APK that performs matrix tests 2–5 (independent BLE
  discovery, GATT services, independent connection, command access) and exports
  a GATT table for `stylo2_ble::recon`. Source builds in Android Studio; its
  pure helpers have JVM unit tests.

## What the prototype can demonstrate, by access situation

The prototype is **honest by construction** — what it shows depends on what the
device actually permits:

| Situation | Expected observable behaviour |
| --- | --- |
| Ordinary install, stock non-root (most likely) | Service is detected as present; bind yields `PermissionDenied` (IOT not granted) → UI shows "Permission denied", controls disabled. **No fake success.** |
| Accessory service absent / other device | `ServiceUnavailable` → UI shows it; fallback offered |
| Direct-BLE protocol verified + attached | Pen detected, profiles selectable, start/stop drive the pen motor |
| Vendor-authorised (official SDK) binding | Full binder route: profiles + start/stop drive the pen motor |
| Fallback engaged | Tablet motor buzzes on start/stop; UI clearly states "tablet, not pen"; no profiles |

The first row is the expected result of simply installing the app the way the
specification requires (own certificate, no root). Demonstrating the pen's own
haptics requires one of the proven routes in `B-feasibility.md`.

## Physical-device test matrix — **NOT YET RUN (requires OnePlus Pad 3)**

Record each result with its method tag: `inspection` / `static` / `emulator` /
`adb` / `physical`. Do not mark a row passed from a successful build.

| # | Test | Result | Method |
| --- | --- | --- | --- |
| 1 | Detect connected Stylo 2 | ☐ | |
| 2 | Independent BLE discovery works | ☐ | |
| 3 | Identify available GATT services | ☐ | |
| 4 | Independent connection possible | ☐ | |
| 5 | Haptic commands accessible (which route?) | ☐ | |
| 6 | Each vibration profile (0–3) → which feel | ☐ | |
| 7 | Start / stop behaviour | ☐ | |
| 8 | Disconnect / reconnect handling | ☐ | |
| 9 | App backgrounding / resume | ☐ | |
| 10 | Behaviour after tablet reboot | ☐ | |
| 11 | Normal stylus input (MotionEvent) still works | ☐ | |
| 12 | OnePlus Notes / system haptics undisturbed | ☐ | |

Tests 11–12 are the non-interference guarantees: the library must never degrade
existing stylus input or system haptics. They are mandatory before any claim of
success.

## How to run the prototype on device (once built)

1. Build the AAR + native libs (D §9) and install the example app with its own
   debug/release certificate — **no root, no Magisk, no priv-app**.
2. Open the app: it reports connection/capability state.
3. If a route is available, pick a profile and toggle feedback; otherwise the
   app shows the precise unavailable/unauthorised reason.
4. Run the matrix above and fill in results with method tags.
