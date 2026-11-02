# Stylo2 Haptics

A Magisk module and companion Android app that enable OnePlus Stylo 2 haptic feedback in every app, not just the ones OnePlus supports.

On stock firmware, the stylus only vibrates in the built-in Notes app and a handful of approved third-party apps (such as HiPaint). This project lets the user turn on the same writing feedback system-wide.

Discussion and install instructions: [XDA Forums thread](https://xdaforums.com/t/magisk-module-app-oneplus-stylo-2-global-haptic-feedback.4758335/)

## Background

Stylus haptics are handled by OnePlus's accessory service, `com.oplus.ipemanager/.btadsorb.CoreService`, which talks to the pen over Bluetooth. Apps reach it by binding with the intent action `com.oplus.ipemanager.ACTION.PENCIL_SDK`, but the service is protected by the restricted system permission `com.oplus.permission.safe.IOT`, which ordinary apps cannot be granted.

- The stock Notes app holds that permission and binds directly through a bundled SDK.
- Approved third-party apps go through a separate OnePlus developer SDK that authenticates each app with OnePlus.
- Any other app is refused.

## How it works

1. **Privileged install.** The Magisk module installs the companion app to `/system/priv-app` and adds a `privapp-permissions` allowlist to the `system`, `system_ext`, and `product` partitions, so Android grants it `com.oplus.permission.safe.IOT` at boot. Nothing in the firmware is modified.
2. **SDK interface.** The app reproduces the service's AIDL interface (`ISdkAidlInterface`) and the Notes app's vibration profiles, taken from analysis of the stock Notes app.
3. **Binding.** `StylusVibrationClient` checks the permission, binds to the service the same way Notes does, and confirms the binder's interface descriptor before using it. Connection state and errors (permission denied, service unavailable, wrong interface) are exposed as a `StateFlow` and shown in the UI.
4. **Feedback.** The user picks a vibration preset (pencil, eraser, ballpoint pen, pen) and turns haptics on. The app calls `setVibrationType` and `startFeedBackVibration`, and the service handles the pen communication.

The app does not send Bluetooth packets itself. It only talks to the system service.

## Installation

Requires a rooted OnePlus device with Magisk.

1. Download `module.zip` from the [latest release](https://github.com/YeetingWaterbottle/stylo2-haptics/releases/latest).
2. Install it in Magisk and reboot.
3. Open **Stylo2 Haptics**, tap **Bind**, choose a vibration preset, and turn on haptics.

To uninstall, remove the module in Magisk and reboot.

## Compatibility

Tested only on a OnePlus Pad 3 running Global OxygenOS (converted from OPD2413 ColorOS to OPD2415), Android 16. Other OnePlus/Oppo devices with the same accessory service may work. Reports from other devices are welcome in the XDA thread.

## Building

Open the project in Android Studio and build the `app` module, or run:

```bash
./gradlew assembleRelease
```

Kotlin, minimum SDK 30, target SDK 36.

## Research

The research behind this project now lives in [Achos0190/Stylo-db](https://github.com/Achos0190/Stylo-db), under `research/`. It covers:
- a structured database of sources, devices, interfaces and open questions on stylus haptics on Android;
- the pen attach-handshake and firmware analysis;
- reference copies of the community regional-ROM stylus fixes;
- a pickup prompt for reading firmware off the tablet.
