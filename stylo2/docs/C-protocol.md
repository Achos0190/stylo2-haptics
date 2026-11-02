# Deliverable C — Protocol specification

Two transports are documented: the **binder/AIDL** surface (reported, from the
in-repo reproduction) and the **direct-BLE** surface (unknown — a recon
procedure, not results). Confirmed, reported, and hypothetical items are kept
strictly separate.

---

## Part 1 — OnePlus accessory-service binder/AIDL surface

### 1.1 Binding (reported)

| Item | Value | Status |
| --- | --- | --- |
| Package | `com.oplus.ipemanager` | VERIFIED in-repo |
| Intent action | `com.oplus.ipemanager.ACTION.PENCIL_SDK` | VERIFIED in-repo |
| Service component | `com.oplus.ipemanager/.btadsorb.CoreService` | VERIFIED in-repo |
| Gating permission | `com.oplus.permission.safe.IOT` | VERIFIED in-repo; level REPORTED (≥privileged) |
| Interface descriptor | `com.oplus.ipemanager.sdk.ISdkAidlInterface` | REPORTED |

A client must confirm `binder.interfaceDescriptor == <descriptor>` before
trusting the proxy (the existing client does this; the Rust `BinderTransport`
contract requires it JVM-side).

### 1.2 Command surface (reported, from in-repo reproduction)

`ISdkAidlInterface`, transaction codes as reproduced:

| Code | Method | Direction | Used by this library |
| --- | --- | --- | --- |
| 1 | `getPencilConnectState() -> int` | query | optional (state) |
| 2 | `getVibrationSwitchState() -> boolean` | query | optional |
| 3 | `isFunctionVibrationEnable() -> boolean` | query | optional |
| 4 | `isDemoModeEnable() -> boolean` | query | no |
| 5 | `enableDemoMode(boolean)` | control | no |
| 6 | `getSuportSdkVersion() -> int` *(sic)* | query | optional |
| 7 | `setVibrationType(int)` | control | **yes** (profile select) |
| 8 | `startVibration(int)` | control | no (prefer 11/12) |
| 9 | `setISdkAidlCallback(cb)` | control | yes (events) |
| 10 | `unsetISdkAidlCallback(cb)` | control | yes |
| 11 | `startFeedBackVibration()` | control | **yes** (start) |
| 12 | `stopFeedBackVibration()` | control | **yes** (stop) |

This library deliberately uses the **minimal** control subset: `setVibrationType`
+ `startFeedBackVibration` + `stopFeedBackVibration`, plus the callback
registration. `enableDemoMode`/`startVibration` are not exposed.

### 1.3 Profiles (reported — verify before trusting)

| Profile (abstract) | Vendor enum | Reported `setVibrationType` int |
| --- | --- | --- |
| Pencil | `PENCIL` | 0 |
| Eraser | `ERASER` | 1 |
| Ballpoint | `BALLPEN` | 2 |
| Pen | `PEN` | 3 |
| *(feedback mode, not a profile)* | `FUNCTION_VIBRATION` | 4 |

**Status: REPORTED / inferred from enum order.** Isolated in
`stylo2-oplus::wire::reported_code`. Do not trust the integers until verified
(§1.5).

### 1.4 Events (reported) — `ISDKAidlCallback`

| Code | Callback | Core event |
| --- | --- | --- |
| 1 | `onConnectionChanged(int)` | `StylusEvent::Connected`/`Disconnected` |
| 2 | `onVibrationSwitchStateChange(boolean)` | `VibrationSwitchChanged(bool)` |
| 3 | `onFunctionFeedbackStateChange(boolean)` | `FunctionFeedbackChanged(bool)` |
| 4 | `onDemoModeEnableChange(boolean)` | *(not surfaced)* |

Button / double-tap / battery events: **not present** in this interface. Treat
as HYPOTHETICAL; do not expose a `StylusEvent` variant for them until a backend
demonstrates them.

### 1.5 On-device verification procedure (owner's own device)

These are read-only inspections the device owner runs on hardware they own, to
turn REPORTED → VERIFIED. They do not modify the system.

- **Permission level:**
  `adb shell dumpsys package com.oplus.ipemanager | sed -n '/declared permissions/,/^$/p'`
  and inspect the framework permission XML
  (`/system/etc/permissions`, `/system_ext/etc/permissions`,
  `/vendor/etc/…`) for `com.oplus.permission.safe.IOT`'s `protectionLevel`. A
  `signature` level means even a privileged install needs the vendor key.
- **AIDL reality check:** confirm the service exists and its descriptor:
  `adb shell dumpsys activity services com.oplus.ipemanager`. For the exact
  method/transaction mapping, inspect the interface as declared by the installed
  accessory APK on the owner's device (interoperability analysis — see
  `LEGAL.md`).
- **Profile integers:** with an *authorised* binding (see B), call
  `setVibrationType(n)` for `n = 0..4`, start feedback, and record which physical
  feel each `n` produces. Correct `reported_code` from the result.

---

## Part 2 — Direct BLE surface (UNKNOWN — recon procedure only)

**No UUIDs, characteristics, or encodings are known. None are invented here.**
The direct-BLE route is viable only if all of the following are established on
the device; each is currently HYPOTHETICAL:

1. the pen is observable to an ordinary app (address/handle);
2. its GATT services are discoverable by a second client while bonded to the
   system;
3. a **writable** characteristic drives the haptic motor;
4. the write payload encoding is known;
5. a second GATT connection does **not** disrupt the system's own connection.

### 2.1 Reconnaissance procedure (on the owner's device)

The **Stylo2 BLE Inspector** app (`android/stylus-ble-inspector`) automates
steps 3–5 below from an ordinary non-root app: scan (with pen candidates
flagged), connect, discover the full GATT tree, read / write / subscribe, and
export the result. Steps 1–2 (observing the system link and capturing its
traffic) remain `adb` / Wireshark work.

1. **Observe the link:** `adb shell dumpsys bluetooth_manager` — find the pen's
   address and current bonding/connection state.
2. **Capture traffic:** enable *Bluetooth HCI snoop log* (Developer options),
   reproduce haptics in OnePlus Notes, pull
   `/sdcard/…/btsnoop_hci.log`, and inspect ATT writes in Wireshark to see which
   handle/characteristic changes around a haptic event.
3. **Independent discovery:** with a generic GATT explorer (e.g. nRF Connect),
   attempt service/characteristic discovery; note whether a second connection is
   permitted and whether the system link survives.
4. **Correlate:** map the characteristic seen in (2) to the discovery in (3);
   derive the command bytes for set-profile / start / stop.
5. **Non-interference test:** confirm normal pen input (MotionEvent) and OnePlus
   Notes haptics still work with the experimental app connected.

### 2.2 Where the result goes

Feed the inspector's exported GATT table into
`stylo2_ble::recon::candidate_control_characteristics` to rank the writable
characteristics as candidate control attributes (a vendor/non-SIG UUID with a
sibling notify channel ranks highest). Then a *verified* result populates
`stylo2_ble::Stylo2GattProfile`:

```rust
Stylo2GattProfile {
    service: /* 16-byte UUID */,
    control_characteristic: /* 16-byte UUID */,
    encode: |cmd| /* verified bytes for SetProfile/Start/Stop */,
}
```

Until that struct is constructed from verified values, `BleBackend` advertises
no capabilities and returns `Unsupported` for every command — by design.

---

## Part 3 — Extended haptic parameters

The reported interface exposes **only** profile selection + start/stop. It has
**no** parameter for intensity, duration, frequency, waveform, trigger timing,
or custom patterns. These are therefore **not exposed** by the library
(`Capabilities` leaves them `false`). Treat their existence as HYPOTHETICAL
pending evidence of a richer interface (e.g. a BLE characteristic that accepts
such parameters). The spec rule applies: *do not expose controls the hardware
interface does not actually support.*
