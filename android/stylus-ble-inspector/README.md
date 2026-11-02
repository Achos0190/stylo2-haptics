# Stylo2 BLE Inspector

A small, standalone **testing APK** for inspecting the Bluetooth LE side of the
OnePlus Stylo 2 (or any BLE device) from an ordinary, non-rooted app. It is the
on-device tool the feasibility report asks for first
(`../../stylo2/docs/B-feasibility.md`) — it turns the direct-BLE route from
*hypothetical* into *measured*.

It does the core of BlueShark's workflow, scoped to connection inspection:

```
Scan ──► Connect ──► Discover GATT ──► Read / Write / Subscribe ──► Export findings
```

- **Scan** — lists advertising BLE devices with name, address, RSSI, advertised
  service UUIDs, and manufacturer IDs. Likely pens are flagged with a
  `★ stylus candidate` badge (a **name heuristic** only — it never asserts
  identity; no UUIDs/OUIs are hard-coded because none are verified).
- **Connect & Discover** — opens a GATT link and lists services →
  characteristics → descriptors, with decoded properties
  (READ / WRITE / WRITE_NR / NOTIFY / INDICATE).
- **Read / Write / Subscribe** — read a characteristic, write raw hex (with or
  without response), or toggle notifications/indications. Every operation is
  serialised (one outstanding ATT request at a time), timeout-bounded, and
  logged with timestamps.
- **Export** — shares a plain-text evidence bundle (device, full GATT table,
  operation log) via the Android share sheet — the input for populating
  `stylo2_ble::recon` / `Stylo2GattProfile`.

## Attribution

Built on patterns and two files from **[BlueShark](https://github.com/nphil/BlueShark)**
(MIT, © 2026 nphil): `GattOperationQueue` (verbatim) and `BlePermissions`
(trimmed). See [`NOTICE`](NOTICE). BlueShark itself is far more capable (HCI
snoop capture via Shizuku, vendor-app sniffing, Home Assistant export); if you
need those, use BlueShark directly. This app is the minimal, stylus-focused
subset.

## Status

> **Source only — not built in the reconnaissance environment** (no Android
> SDK there). It is a complete, self-contained Gradle project. The pure helpers
> (`HexCodec`, `StylusHeuristics`) have JVM unit tests; the BLE paths require a
> physical device to exercise.

## Build & run

Open `android/stylus-ble-inspector/` in Android Studio, or:

```bash
cd android/stylus-ble-inspector
./gradlew :app:assembleDebug        # build the APK
./gradlew :app:testDebugUnitTest    # run the JVM unit tests
./gradlew :app:installDebug         # install on a connected device
```

`minSdk 33`, `targetSdk 36` — the OnePlus Pad 3 (Android 16) is well within
range, and this lets the app use only the modern, non-deprecated BLE APIs.

## Using it to profile the Stylo 2

1. Pair/connect the pen normally first (OxygenOS manages that link).
2. Open the inspector, grant the nearby-devices permission, **Start scan**, and
   look for the `★ stylus candidate` entry (or match by address from
   `adb shell dumpsys bluetooth_manager`).
3. Connect, **Discover services**, and look for a **writable** characteristic —
   that is the candidate haptic-control attribute.
4. Subscribe to notify/indicate characteristics, then write candidate payloads
   and watch the log for device responses.
5. **Export** and feed the GATT table into `stylo2_ble` to draft a
   `Stylo2GattProfile`. Cross-check against an HCI snoop capture of OnePlus
   Notes (see `../../stylo2/docs/C-protocol.md`) to confirm the real command
   bytes.

> Note: a second GATT connection may or may not coexist with the system's own
> link to the pen — testing whether it interferes is itself one of the
> questions in `docs/C`. Watch for the system link dropping while the inspector
> is connected.
