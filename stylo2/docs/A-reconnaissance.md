# Deliverable A — Technical reconnaissance report

**Date:** 2026-11-02 · **Target:** OnePlus Pad 3 + OnePlus Stylo 2, stock
OxygenOS (Android 16).

Each finding is tagged:

- **[VERIFIED]** — confirmed by me from a primary source (this repository's
  code, or Rust I compiled/ran here). "Verified" here means *verified against
  the artifact named*, not necessarily against the live device.
- **[REPORTED]** — from a community/secondary source, or from this repo's
  *reproduction* of a vendor interface; plausible but not confirmed against the
  actual firmware by me.
- **[HYPOTHETICAL]** — open question; not yet established either way.

> No physical device was available in this environment. Nothing below is a
> physical-device result. On-device procedures to upgrade [REPORTED]/
> [HYPOTHETICAL] items to [VERIFIED] are given in `C-protocol.md` and
> `E-prototype.md`.

## 1. Software interfaces (binder / AIDL route)

- **[VERIFIED]** The existing app binds the OnePlus accessory service with
  intent action `com.oplus.ipemanager.ACTION.PENCIL_SDK`, package
  `com.oplus.ipemanager`, and declares `<uses-permission
  com.oplus.permission.safe.IOT>` plus a `<queries>` entry for the package and
  action. (Source: `app/src/main/AndroidManifest.xml`,
  `OplusPencilSdkClient.kt` in this repo.)
- **[VERIFIED]** The service component referenced is
  `com.oplus.ipemanager/.btadsorb.CoreService`. (Source: repo `README.md`.)
- **[VERIFIED / in-repo reproduction]** The repo reproduces two AIDL-style
  interfaces (`app/src/main/java/com/oplus/ipemanager/sdk/`):
  - `ISdkAidlInterface` (descriptor `com.oplus.ipemanager.sdk.ISdkAidlInterface`)
    with transaction codes 1–12 and methods: `getPencilConnectState()`,
    `getVibrationSwitchState()`, `isFunctionVibrationEnable()`,
    `isDemoModeEnable()`, `enableDemoMode(boolean)`, `getSuportSdkVersion()`
    *(sic)*, `setVibrationType(int)`, `startVibration(int)`,
    `setISdkAidlCallback(cb)`, `unsetISdkAidlCallback(cb)`,
    `startFeedBackVibration()`, `stopFeedBackVibration()`.
  - `ISDKAidlCallback` (descriptor `com.oplus.ipemanager.sdk.ISDKAidlCallback`)
    with codes 1–4: `onConnectionChanged(int)`,
    `onVibrationSwitchStateChange(boolean)`,
    `onFunctionFeedbackStateChange(boolean)`, `onDemoModeEnableChange(boolean)`.
- **[REPORTED]** That these signatures, transaction codes and the descriptor
  string match the **actual** service on the target firmware. The repo files
  are a community reproduction ("cleaned"); they must be re-confirmed against
  the installed `com.oplus.ipemanager` (see C-protocol §"Verifying the AIDL").
- **[VERIFIED / in-repo]** The client confirms the binder's interface
  descriptor equals `com.oplus.ipemanager.sdk.ISdkAidlInterface` before use, and
  surfaces permission-denied / service-unavailable / wrong-interface distinctly.
  (Good defensive practice; preserved in the Rust design.)

## 2. Vibration profiles

- **[VERIFIED / in-repo]** A vendor-style enum `Vibration { PENCIL, ERASER,
  BALLPEN, PEN, FUNCTION_VIBRATION }` exists, and the client passes
  `enum.ordinal()` to `setVibrationType(int)`.
- **[REPORTED]** Therefore the wire values are `PENCIL=0, ERASER=1, BALLPEN=2,
  PEN=3, FUNCTION_VIBRATION=4`. This is **inferred from declaration order**,
  which the specification warns against trusting. Isolated in
  `stylo2-oplus::wire::reported_code` for single-point correction after
  on-device verification.

## 3. Permissions

- **[VERIFIED / in-repo]** The route depends on `com.oplus.permission.safe.IOT`.
- **[REPORTED/INFERRED]** Its protection level is restricted — at least
  `privileged` (not grantable to an ordinary installed app). Evidence: (a) the
  existing project only obtains it by installing to `/system/priv-app` with a
  `privapp-permissions` allowlist, which is how Android grants *privileged*
  permissions; (b) public system-app permission dumps list the name on several
  `com.oplus.*` system packages but publish no level for it.
- **[HYPOTHETICAL]** Whether a *signature* check (vendor platform key) is also
  involved, which would make even a privileged install insufficient without the
  right signer. To be determined on-device (C-protocol §"Verifying the
  permission").
- **[VERIFIED]** Standard runtime permissions relevant to the BLE route:
  `BLUETOOTH_CONNECT`, `BLUETOOTH_SCAN` (Android 12+). These *are* grantable to
  an ordinary app with user consent.

## 4. Bluetooth / BLE

- **[REPORTED]** The pen connects over Bluetooth/BLE and is managed by the
  system; its haptic motor is physically in the pen. (Retailer specs describe
  pressure + haptic feedback in the pen; repo states the service "talks to the
  pen over Bluetooth".)
- **[HYPOTHETICAL]** All of: whether the pen advertises/open GATT services to a
  second (non-system) client; whether its address is observable to an app;
  whether a *writable* characteristic triggers haptics; whether the command
  encoding is discoverable; whether a second connection disrupts the system
  link; whether Companion Device Manager helps. None established. Recon
  procedure in C-protocol §BLE.

## 5. Official SDK / developer program

- **[VERIFIED — negative]** A web search (2026-11-02) surfaced **no public,
  official OnePlus/OPPO Pencil SDK, AAR/JAR, or third-party integration
  documentation**. The most relevant hit is the project's own XDA thread.
- **[REPORTED]** Community claim (XDA): the only non-root paths are (1) obtain a
  OnePlus developer API, or (2) reverse-engineer the Bluetooth packets. Approved
  third-party apps reportedly authenticate per-app through a OnePlus developer
  SDK; the stock Notes app holds the IOT permission directly.
- **[HYPOTHETICAL]** Whether such a developer program is open to independent
  developers, what registration/terms/signing-certificate/token requirements it
  imposes, and whether it covers the Pad 3 + Stylo 2. **Do not assume an
  endpoint or credentials exist.** Action: contact OnePlus/OPPO developer
  relations; see B-feasibility.

## 6. Known community implementation (this repository)

- **[VERIFIED / in-repo]** Architecture: a Compose/View app + `StylusVibrationClient`
  that binds the service and calls `setVibrationType` + `startFeedBackVibration`
  / `stopFeedBackVibration`; it does **not** send Bluetooth packets itself.
- **[VERIFIED / in-repo]** Access mechanism: a **Magisk module** installs the
  app to `/system/priv-app` and adds `privapp-permissions` allowlists so Android
  grants the IOT permission at boot.
- **[VERIFIED / in-repo]** Compatibility claim: tested only on a OnePlus Pad 3,
  Global OxygenOS (converted OPD2413→OPD2415), Android 16.
- **Key distinction (central to this project):** the *haptic-control* logic
  (the AIDL calls) is reusable and legitimate; the *privileged-install*
  mechanism (Magisk, priv-app) is the part this project must **not** depend on.
  The two are cleanly separable, and this workspace keeps only the former.

## 7. Source inventory

| Source | Type | Used for |
| --- | --- | --- |
| This repo (`AndroidManifest.xml`, `com/oplus/ipemanager/sdk/*.java`, `OplusPencilSdkClient.kt`, `README.md`) | Primary (in-repo) | §1–§3, §6 |
| XDA thread *OnePlus Stylo 2 Global Haptic Feedback* (`xdaforums.com/t/...4758335`) | Community | §3, §5 |
| sysapps.bayton.org `com.oplus.*` package permission listings | Secondary | §3 (name present, level absent) |
| Web search, 2026-11-02 (official SDK existence) | Secondary | §5 |
| Retailer/spec listings (Stylo 2 pressure + haptics) | Secondary | §4 |

**Not performed:** downloading/decompiling the proprietary Smart Accessories
APK. The AIDL is already reproduced in-repo from community analysis; re-deriving
it from the binary is unnecessary for this stage and is better done, if needed,
under the interoperability framing in `LEGAL.md` and against the APK actually
installed on the target tablet.
