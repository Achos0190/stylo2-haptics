# Deliverable I — Source & evidence ledger

Every load-bearing claim, its status, its source, and exactly how to upgrade it.
Keep this current as device work lands — it is the project's single honesty
record. Status: **[V]erified** (confirmed by me from the repo or a primary
source, i.e. against the artifact named — *not* the live device unless stated),
**[R]eported** (community/secondary, or this repo's reproduction of a vendor
interface), **[H]ypothesis** (open).

> No claim below is a physical-device result. On-device confirmation lives in
> `E-prototype.md` (matrix) and `J-capture-plan.md` (captures), tagged by method.

## Scope (owner-confirmed)

| Fact | Status | Implication |
| --- | --- | --- |
| On this device everything the stylus does works **except** haptics in third-party apps: detection, pressure, tilt, hover, normal input, the BT bond, and haptics **inside OnePlus Notes** | [V] owner-reported; Notes demonstrably vibrates the pen | The actuator, the haptic command path, and the link are **proven good**. Pen input is an open uplink (standard Android stylus/HID); haptics is a gated downlink control command. The only missing piece is a **third-party app's authorization** to issue that command — an authorization wall on a working capability, **not** a broken transport, unpaired device, or incapable pen. Weight the investigation on **B1/B2** (permission / caller validation); the pen, bond, and actuator are not suspects. Route B narrows to "can a second app reach the same working command path without disrupting the system link," and an HCI snoop of Notes is **guaranteed** to contain the real haptic bytes. Do not touch pen input. |

## Service, binding, AIDL

| Claim | Status | Source | Upgrade to [V-device] by |
| --- | --- | --- | --- |
| Binds `com.oplus.ipemanager` via action `...ACTION.PENCIL_SDK` | [V] in-repo | `AndroidManifest.xml`, `OplusPencilSdkClient.kt` | n/a (code fact) |
| Service component `com.oplus.ipemanager/.btadsorb.CoreService` | [V] in-repo (README) | repo README | `dumpsys activity services com.oplus.ipemanager` on device |
| Interface descriptor `com.oplus.ipemanager.sdk.ISdkAidlInterface` | [R] | repo reproduction | confirm against installed APK (JADX) + descriptor at bind |
| Methods: `getPencilConnectState`, `getVibrationSwitchState`, `isFunctionVibrationEnable`, `setVibrationType(int)`, `startFeedBackVibration`, `stopFeedBackVibration`, `set/unsetISdkAidlCallback`, … (txn codes 1–12) | [R] | repo reproduction | JADX of the installed `com.oplus.ipemanager` of the exact firmware version |
| Callback `ISDKAidlCallback`: `onConnectionChanged`, `onVibrationSwitchStateChange`, `onFunctionFeedbackStateChange`, `onDemoModeEnableChange` | [R] | repo reproduction | same |
| Service is exported / bindable by a third-party app | [H] | — | isolating experiment step 1–2 (`H-trust-boundaries.md`) |

## Permissions & authorization

| Claim | Status | Source | Upgrade by |
| --- | --- | --- | --- |
| Route is gated by `com.oplus.permission.safe.IOT` | [V] in-repo | manifest | n/a |
| That permission is `≥privileged` (not grantable to an ordinary app) | [R]/inferred | existing project needs priv-app install; sysapps listings show the name w/o level | `dumpsys package <perm>` + framework permission XML `protectionLevel` (G §3) |
| A **signature** check (vendor platform key) is also required | [H] | — | read `protectionLevel` flags; if `signature`, confirm signer family (B0) |
| The service additionally validates caller UID/package/signature *after* bind | [H] | — | isolating experiment step 3 (bind succeeds, call fails) |
| Approved third-party apps authenticate per-app via a OnePlus developer SDK | [R] | XDA thread | official OnePlus/OPPO developer-relations enquiry |

## Profiles & commands

| Claim | Status | Source | Upgrade by |
| --- | --- | --- | --- |
| Four profiles: Pencil, Eraser, Ballpoint(=BALLPEN), Pen | [R] | repo `Vibration` enum | capture/observe each selection (J scenarios 2–5) |
| Wire values `setVibrationType`: Pencil=0, Eraser=1, Ballpoint=2, Pen=3 | [R], inferred from enum order | `stylo2-oplus::wire` | call `setVibrationType(0..4)` on device (authorised binder) or read bytes from snoop; feel the pen |
| Interface exposes **no** intensity/duration/frequency/waveform/custom params | [R] | reproduced AIDL shape | JADX of installed APK; snoop for richer framing |

## Bluetooth / transport

| Claim | Status | Source | Upgrade by |
| --- | --- | --- | --- |
| Pen is connected/managed by the system over Bluetooth; motor is in the pen | [R] | repo, retailer specs | `dumpsys bluetooth_manager` |
| Haptic control rides BLE **GATT** (writable characteristic) | [H] | — | HCI snoop of Notes (J); the `.btadsorb` name warns it may not |
| Pen exposes GATT to a *second* (non-system) client | [H] | — | inspector connect while bonded (G §6) |
| A second GATT connection does **not** disrupt the system link / Notes | [H] | — | inspector connected + draw in Notes (E 11–12) |
| B4 requires a session handshake / app-level auth before haptics | [H] | — | look for a pre-haptic exchange in the snoop (J) |

## Official SDK

| Claim | Status | Source | Upgrade by |
| --- | --- | --- | --- |
| No public OnePlus/OPPO Pencil SDK, AAR/JAR, or 3rd-party integration docs | [V] (negative, as of 2026-11-02) | web search | re-check developer portal; contact developer relations |

## Host-side (this repo)

| Claim | Status | Source |
| --- | --- | --- |
| `stylo2` workspace compiles; 26 tests pass; clippy clean (x86_64) | [V] | `cargo test`/`clippy` run in this session |
| `jni` feature type-checks against `jni 0.21` | [V] | `cargo check --features jni` |
| Android modules (`stylo2-bridge`, `stylus-ble-inspector`) compile | [H] | **not built — no Android SDK in the authoring env** |

## Open questions → the exact experiment that answers each (deliverable 9)

| # | Open question | Exact experiment | Feeds |
| --- | --- | --- | --- |
| Q1 | Protection level of `safe.IOT`? | `dumpsys package com.oplus.permission.safe.IOT`; framework XML `protectionLevel` | B1; decides if binder route is possible at all |
| Q2 | Can an ordinary app discover + bind the service? | Isolating experiment steps 1–2 (H) | B2 |
| Q3 | If bind succeeds, do calls pass caller validation? | Isolating experiment step 3 (H) | B2 vs B4 |
| Q4 | Is haptic control carried over GATT, or another channel? | HCI snoop of Notes, one "pencil" stroke + start/stop (J §4) | B4; picks BLE vs not |
| Q5 | Exact bytes per profile and for start/stop? | Snoop scenarios 2–7, correlate markers↔ATT writes (J) | `Stylo2GattProfile.encode`, `wire::reported_code` |
| Q6 | Can a 2nd app connect to the pen's GATT? | Inspector connect while system connected (G §6) | B3 |
| Q7 | Does a 2nd connection disrupt the system link/Notes? | Inspector connected + draw in Notes | B3; E 11–12 (gating) |
| Q8 | Is there a session handshake before the first haptic? | Inspect snoop scenario 9 (initial connection) | B4 |
| Q9 | Does writing a captured value to the candidate characteristic buzz the pen? | Inspector write of the J-captured bytes | the go/no-go for the direct-BLE route |
| Q10 | Do the reproduced AIDL signatures match the installed firmware? | JADX the installed `com.oplus.ipemanager` of this exact OxygenOS 16 build | upgrades the AIDL rows above to [V] |
| Q11 | Is there a legitimate official developer-authorisation path? | OnePlus/OPPO developer-relations enquiry | the only non-root binder route if B1/B2 are closed |
