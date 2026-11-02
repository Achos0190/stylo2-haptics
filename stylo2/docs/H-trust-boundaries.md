# Deliverable H — Architecture & trust-boundary report

What sits between a third-party app and the pen's haptic motor, where each
authorization decision is enforced, and the experiment that isolates **which
boundary actually blocks third-party haptics**. Classifications: **[V]**erified
(by me from repo/primary source), **[R]**eported (community/secondary), **[H]**
ypothesis (open).

## Architecture (two candidate paths to the motor)

```
                 THIRD-PARTY APP (own certificate, no root)
                        │                         │
            binder path │                         │ direct-BLE path
                        ▼                         │
          ┌───────── B1 ─────────┐                │
          │ Android framework    │                │
          │ permission manager:  │                │
          │ com.oplus.permission │                │
          │ .safe.IOT  [R: ≥priv]│                │
          └──────────┬───────────┘                │
                     ▼                             │
          ┌───────── B2 ─────────┐                 │
          │ com.oplus.ipemanager │                 │
          │ .btadsorb.CoreService│                 │
          │ Binder caller check? │                 │
          │ (UID/pkg/signature)  │                 │
          │   [H]                │                 │
          └──────────┬───────────┘                 │
                     ▼                              ▼
          ┌──────────────────────── B3 ───────────────────────┐
          │  System Bluetooth stack (bond + link encryption)   │
          │  owns the live connection to the pen   [R]         │
          └──────────────────────────┬─────────────────────────┘
                                      ▼
          ┌──────────────────────── B4 ───────────────────────┐
          │  Vendor protocol framing + session init / app-level │
          │  authentication over the link          [H]          │
          └──────────────────────────┬─────────────────────────┘
                                      ▼
                           ┌──────────────────────┐
                           │  OnePlus Stylo 2      │
                           │  haptic motor         │
                           └──────────────────────┘
```

The binder path (left) is how OnePlus Notes drives the motor; it crosses **B1**
and **B2** and then the system owns **B3/B4**. The direct-BLE path (right) tries
to reach **B3/B4** from an independent app without B1/B2 — viable only if the
pen accepts a second client and the framing is reproducible and not
app-authenticated. **It is unknown whether B4 even rides GATT** (the service is
`.btadsorb.*`); `J-capture-plan.md` settles that.

## Trust-boundary table

| ID | Mechanism | Enforcement point | What an ordinary app faces | Status | Experiment to pin it down |
| --- | --- | --- | --- | --- | --- |
| **B0** | APK signing identity | OnePlus build signs the vendor apps; Android verifies at install | The app can only present its *own* cert; it is not in OnePlus's signing family | [R] | `apksigner verify --print-certs` on the vendor APK vs the app; compare signers (identification only — a public cert is not the private key) |
| **B1** | `com.oplus.permission.safe.IOT` grant | Android permission manager (install/runtime), gated by the permission's `protectionLevel` | If level is `signature`/`privileged`, grant is **refused** to an ordinary install | [R] (≥privileged inferred from the priv-app requirement) | `dumpsys package com.oplus.permission.safe.IOT`; locate the `<permission>` decl in framework XML; read `protectionLevel` (G §3) |
| **B2** | Binder caller validation in `CoreService` | Inside the service process (may check UID/package/signature) and/or a `<service android:permission=>` | bind may fail, or succeed-then-reject calls; the *error layer* tells you which | [H] | Separate tests below: bind, then call; record the exact exception/layer |
| **B3** | BT pairing/bond + LE link encryption | System Bluetooth stack | The pen is already bonded to the system; a second app opening its own GATT may or may not be allowed, and may disrupt the system link | [H] | Inspector connect while system connected; watch `dumpsys bluetooth_manager` and whether Notes keeps buzzing (E 11–12) |
| **B4** | Vendor protocol framing + any session/app-level auth | On the link, between the vendor service and the pen | Even with a GATT write path, the pen may require a session handshake or per-command auth | [H] | HCI snoop of Notes across the 9 scenarios; look for a handshake preceding the first haptic (J) |

## Which boundary actually blocks? — the isolating experiment

Because OnePlus Notes drives the motor end-to-end today, the actuator, the
command path, and the link are known-good — so **B1/B2 (authorization) are the
prime suspects**, and B3/B4 are questions of *sharing or replicating* a path
that already works, not of whether it works at all. (Pen input is a separate,
open uplink and is out of scope.)

The mission requires separating **discovery**, **binding**, **authorization**,
and **execution**. Run these four as distinct tests (binder path) and read the
result off the *first* one that fails:

1. **Discovery** — can an ordinary app see the service? `PackageManager`
   `queryIntentServices` for `com.oplus.ipemanager.ACTION.PENCIL_SDK`.
   - Fails → the service isn't exported/queryable to third parties (B2, export).
2. **Binding** — does `bindService` return true and `onServiceConnected` fire?
   - `bindService` throws `SecurityException` / returns false → **B1** (the IOT
     permission) or a bind-time `android:permission` on the service.
3. **Authorization** — with the binder in hand, does a *query* call
   (e.g. connection-state) succeed, or throw `SecurityException`/return an auth
   error?
   - Fails here but bind succeeded → **B2** caller validation (UID/package/
     signature *inside* the service), **not** the Android permission.
4. **Execution** — does `setProfile` + `startFeedback` actually buzz the pen?
   - Fails here, after auth succeeds → **B4** (protocol/session), not Android.

Record the exact exception, message, and originating layer (framework vs Binder
vs downstream) for each — a permission denial at step 2 says nothing about
whether the *protocol* at B4 is authenticated (mission 5.2). The inspector +
`G`/`J` cover the direct-BLE path (B3/B4) independently.

## Target connection-state machine (grows in as evidence lands)

The code currently ships the honest subset
(`Disconnected, Connecting, Connected, PermissionDenied, ServiceUnavailable`).
The lifecycle below is the shape it widens into once a real session flow is
verified (see `MISSION.md` for why it is not pre-built):

```
 Disconnected ──scan──▶ Discovering ──found──▶ Connecting ──linked──▶ Connected
                                                                        │
                                                            session init│
                                                                        ▼
                                                                  Initialising
                                                                        │ ok
                                                                        ▼
                                                                     Ready ──set_profile/start/stop──▶ (commands)
   any state ──error──▶ Failed{reason}         (PermissionDenied / ServiceUnavailable
   any state ──drop──▶  Disconnected             are today's concrete Failed reasons)
```

Only `Ready` accepts haptic commands; the `HapticManager` guard that today
keys on `Connected` moves to `Ready` when `Initialising` becomes a real step.
