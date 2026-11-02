# Deliverable G — Local continuation guide (OnePlus Pad 3 over ADB)

How to pick this project up on a local machine with the Pad 3 attached by ADB,
and drive it to a verified haptic command. It turns the open questions in
`B-feasibility.md` / `C-protocol.md` into a concrete, ordered procedure.

> Everything in the repo so far is **host-verified only** (`cargo test`) — no
> device work has been done. This guide is the device half. Record every result
> with a method tag (inspection / static / adb / physical) in the matrix in
> `E-prototype.md`.

---

## 0. The one fact that shapes the whole workflow

The pen's haptics work today **only** through OnePlus Notes (and a few approved
apps), which drive the accessory service `com.oplus.ipemanager` over the
**system's** Bluetooth link. Two different observation layers matter, and they
see different things:

| You want to… | Use | Why |
| --- | --- | --- |
| See **what Notes sends** to make the pen buzz | **HCI snoop log** (btsnoop) | The snoop is below the stack — it captures the *system's* connection. |
| Test whether **your own app** can reach the pen | **Stylo2 BLE Inspector** | A second GATT client has its own connection; it **cannot** see Notes' writes. |

So the inspector is **not** a sniffer. It answers "can an independent app
connect and write, and does that break Notes?" — the HCI snoop answers "what are
the bytes?". You need both.

Also keep an open mind on transport: the service component is `.btadsorb.*`
("adsorb" = the magnetic attach/charge relationship), so the haptic path may not
be a plain GATT write. **The snoop decides** whether it is BLE GATT, a different
L2CAP channel, or something the accessory service brokers. Don't assume GATT
until the capture shows it.

---

## 1. Prerequisites (local machine)

- **Rust**: `rustup`, plus `rustup target add aarch64-linux-android` and
  `cargo install cargo-ndk` for the native build.
- **Android Studio** with SDK (API 36) and the **NDK**.
- **platform-tools** (`adb`) on PATH.
- **Pad 3**: Developer options on, USB debugging on. Because you need to *draw*
  on the pad while it's wired to the laptop, set up **wireless debugging** too
  (`adb pair` / `adb connect <ip>:5555`, Android 11+) so the tablet is free to
  hold.
- **Shizuku** (recommended): ADB-activated, **no root** — it is what lets
  BlueShark capture the HCI snoop on a stock device. Start it with the command
  the Shizuku app shows under *Start via ADB* (typically
  `adb shell sh /sdcard/Android/data/moe.shizuku.privileged.api/start.sh`).
- **BlueShark** (<https://github.com/nphil/BlueShark>) installed on the pad —
  it has the full Shizuku HCI-snoop capture + marker + decode workflow that this
  project deliberately did not re-implement.
- **Wireshark** on the laptop (for the manual snoop path).

## 2. Get the code and confirm the host baseline

```bash
# from a clone of the repository, on the working branch (see ../../PICKUP.md)
cd stylo2 && cargo test --workspace      # expect 26 passing
cargo run -p stylo2-core --example demo --features mock
cd ..
```

Build and install the inspector (all Android modules share the Gradle build at
the repository root):

```bash
./gradlew :stylus-ble-inspector:testDebugUnitTest   # pure-logic unit tests
./gradlew :stylus-ble-inspector:installDebug        # installs on the connected pad (ask the owner first)
```

The tablet read-out in §3 is automated, read-only, by
`python3 research/tools/pad3_readout.py all` (see `../../PICKUP.md` Step 2); the
commands below are what it runs, kept for reference.

## 3. Device inventory (read-only ADB)

None of this changes the device.

```bash
adb devices

# The pen's address, bonding, and current connection state:
adb shell dumpsys bluetooth_manager | sed -n '/Bonded devices/,/^$/p'

# Installed Smart Accessories version (match it to the APK you analyse):
adb shell dumpsys package com.oplus.ipemanager | grep -i versionName

# Is the accessory service real, and what descriptor does it export?
adb shell dumpsys activity services com.oplus.ipemanager

# PROTECTION LEVEL of the gating permission — the binder route hinges on this:
adb shell dumpsys package com.oplus.permission.safe.IOT
adb shell pm list permissions -d -g | grep -i -A2 'safe.IOT'
# And the framework declaration (signature? privileged?):
adb shell 'for d in /system/etc/permissions /system_ext/etc/permissions /vendor/etc/permissions /product/etc/permissions; do grep -ril "safe.IOT" $d 2>/dev/null; done'
```

Record the protection level in `C-protocol.md` §1.5. If it is `signature`,
even a privileged install needs OnePlus's platform key → the binder route is
closed to an independent app, and BLE or the official SDK are the only options.

## 4. Capture what Notes does (the protocol)

This is the highest-value step: it reveals the real command bytes.

**Option A — BlueShark + Shizuku (recommended, no root).** Start Shizuku, grant
it to BlueShark, and use BlueShark's capture flow: it flips
`persist.bluetooth.btsnooplogmode` to full, restarts Bluetooth, launches the
vendor app, lets you drop **named markers** ("pencil", "eraser", "start",
"stop") while you draw, then collects and decodes the snoop (H4 → ACL → L2CAP →
ATT, with handle→UUID resolution).

**Option B — Developer Options + bugreport (no extra app).**
1. Developer options → *Enable Bluetooth HCI snoop log* → **Enabled**; toggle
   Bluetooth off/on.
2. Draw in Notes, exercising each profile and start/stop; note wall-clock times.
3. `adb bugreport bugreport.zip`, then extract the btsnoop/btsnooz blob (AOSP
   embeds a compressed `btsnooz` in the bugreport; decode with BlueShark's
   decoder or AOSP `btsnooz.py`). Open in **Wireshark**, filter `btatt`.

Either way, for **each** profile and for start/stop, record: the ATT **opcode**
(write / write-command), the **handle**, and the **value bytes**. That table is
the ground truth for both routes.

## 5. The tandem method (your idea), done right

OnePlus Notes is the only known-good haptic driver, so you learn by watching it
— and the Pad 3's split-screen lets you keep it drawing-active while you observe.

```
┌───────────────────────────┬───────────────────────────┐
│  OnePlus Notes (left)      │  Observer (right)          │
│  draw → pen haptics fire   │  BlueShark markers /       │
│  (keep focus here)         │  adb logcat timestamps     │
└───────────────────────────┴───────────────────────────┘
```

- Put **Notes** and the **observer** (BlueShark, or just a `adb logcat -v time`
  terminal you glance at) in split-screen. Large-screen Android keeps both
  panes *resumed*, so Notes keeps receiving pen input while the other pane
  updates — that's exactly why multi-window helps: no app-switch backgrounds
  Notes and interrupts the haptic session.
- Draw with each tool/profile and drop a marker (or note the time) at each
  change. Then line markers up against the ATT writes from §4.
- **First confirm** haptics still fire in split-screen (they should) before
  trusting correlations.

Note the inspector is **not** the right-pane tool for *sniffing* Notes (it can't
see Notes' link). It is the tool for §6.

## 6. Test independent access (Stylo2 BLE Inspector)

Now answer whether an ordinary app can reach the pen at all:

1. Open the inspector, grant nearby-devices, **Start scan**. Find the pen by the
   `★ stylus candidate` badge or by the address from §3.
2. **Connect** and **Discover**. Record the full GATT tree. Export it.
3. Look for a **writable** characteristic. Feed the exported table to the Rust
   ranker:
   ```
   stylo2_ble::recon::candidate_control_characteristics(&discovered)
   ```
   (vendor/non-SIG UUID + a sibling notify channel ranks highest).
4. **Non-interference test (docs/C question 5):** with the inspector connected,
   switch to Notes (split-screen) and draw. Do haptics still work? Does the
   system link drop (watch `dumpsys bluetooth_manager`)? Record the answer — it
   decides whether a second connection is even usable.
5. **The moment of truth:** write the exact bytes you observed from Notes in §4
   to the candidate characteristic (start with the known-safe captured value).
   If the pen buzzes, the **direct-BLE route is real**.

> You are writing to hardware you own; the captured bytes are known-safe because
> Notes just sent them. Don't fuzz blindly — start from observed values.

## 7. Feed results back into the code

- **Direct-BLE confirmed** → fill in `stylo2_ble::Stylo2GattProfile { service,
  control_characteristic, encode }` with the verified UUIDs and the per-command
  byte encoder from §4, write a Kotlin `GattTransport`, and the `BleBackend`
  lights up behind the existing API. Flip `Availability.ble_protocol_verified`.
- **Binder route (only with the restricted permission or official
  authorisation)** → implement the Kotlin `BinderTransport` over the existing
  `StylusVibrationClient`; then verify the profile integers empirically: call
  `setVibrationType(n)` for `n = 0..4`, start feedback, and record which feel
  each `n` produces. Correct `stylo2-oplus::wire::reported_code` from that, and
  mark `C-protocol.md` §1.3 **verified**.
- **Update the docs:** promote confirmed items in `C-protocol.md` from
  *reported/hypothetical* to *verified*, and fill the test matrix in
  `E-prototype.md` with method tags.

## 8. First-30-minutes checklist

1. `adb devices` + wireless debugging working while you hold the pad.
2. §3 device inventory — **especially the `safe.IOT` protection level**.
3. Install the inspector; scan; confirm the pen is detectable (matrix test 1–2).
4. One BlueShark HCI capture of Notes doing a single "pencil" stroke + start/stop
   — just enough to see whether haptic control is even ATT/GATT.
5. From that one capture you'll know which route to invest in. Then go deep.

## 9. Success criteria per route

| Route | "Done" means |
| --- | --- |
| Direct BLE | Inspector writes a captured value to a characteristic, pen buzzes, Notes still works with the inspector connected. |
| Binder | App holds `safe.IOT` (via official authorisation), binds the service, `setVibrationType`+`startFeedBackVibration` buzz the pen; profile ints verified. |
| Fallback | Tablet vibrator buzzes on start/stop — already implemented, clearly labelled "not the pen". |

## 10. Boundaries (unchanged on-device)

Read-only `dumpsys`; no system modification; Shizuku is ADB-activated, **not
root**; the only state-changing step is writing bytes to the pen in §6.5 —
start from observed-safe values. Do not extract or reuse vendor keys. If a route
needs the restricted permission and you don't hold it, stop and report
`PermissionDenied` honestly — don't reach for the privileged/Magisk install this
project excludes.
