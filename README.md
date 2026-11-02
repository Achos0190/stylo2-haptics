# Stylo-db: open Stylo 2 haptics research

An open-source effort to let any Android app use the haptic motor in the **OnePlus Stylo 2** stylus, together with a public **database** that records what is known about how those haptics work, where each fact came from, and what is still unknown.

It is done **without cracking**. Nothing here defeats a signature, permission or app-authentication check, extracts vendor keys or tokens, patches OnePlus apps or firmware, or redistributes OnePlus binaries. The work observes hardware the owner already has, documents what it finds, and builds independent code on top of interfaces it is allowed to use. If no such interface exists, the honest result is a documented "no, and here is exactly why".

## Basis

On a OnePlus Pad 3 everything the Stylo 2 does already works in every app: detection, pressure, tilt, hover, normal input and the Bluetooth bond. Its **haptics** work only in OnePlus Notes and a few approved apps.

The difference is direction:

- **Pen input** travels from the pen to the tablet. It uses Android's standard stylus stack, so every app gets it.
- **Haptics** are a control command from the tablet to the pen. OnePlus routes it through its accessory service `com.oplus.ipemanager` (action `com.oplus.ipemanager.ACTION.PENCIL_SDK`), which is protected by the restricted permission `com.oplus.permission.safe.IOT`.

So the motor, the command path and the Bluetooth link all work. What a third-party app lacks is **authorisation** to send the command. This rests on the owner's report that haptics fire in Notes, and the first on-device session confirms it. The question this project answers is whether a legitimate, open route to that command exists.

## Scope

**In scope**

| Route | What it tests | Where |
| --- | --- | --- |
| A. Unprivileged binder | Can an ordinary app bind the pen service and call its haptic methods? The app reports which step the platform refuses, if any | [`android/stylo2-poc`](android/stylo2-poc) |
| B. Direct Bluetooth LE | Does the pen accept an independent connection, and do the bytes OnePlus Notes sends on the owner's own device make it vibrate when an app sends them? | [`android/stylus-ble-inspector`](android/stylus-ble-inspector), [`stylo2/docs/J-capture-plan.md`](stylo2/docs/J-capture-plan.md) |
| C. Official authorisation | OnePlus's developer SDK for approved apps | Documented in [`stylo2/docs/B-feasibility.md`](stylo2/docs/B-feasibility.md) |
| Fallback | The tablet's own vibrator, clearly labelled "not the pen" | [`stylo2/`](stylo2) |

All of this sits behind one open, backend-neutral API (the Rust workspace in [`stylo2/`](stylo2) with a Kotlin bridge), so apps don't have to care which route turns out to work.

**Supporting research**: how the pen pairs in the first place (the CPS8601 magnetic charger chip, its firmware and the regional-ROM pairing fix), because nothing works on the pen until that handshake succeeds.

**Out of scope**

- Bypassing `safe.IOT`, the service's caller checks or OnePlus's app authentication.
- Root, Magisk, a `/system/priv-app` install or firmware changes as the way the haptics are delivered. The upstream app in [`app/`](app) does exactly this. It is kept as the reference for the service interface, but it is not the route this project builds. Research may *read* from a tablet that happens to be rooted (see [`PICKUP.md`](PICKUP.md)), but it never writes to it.
- Re-implementing pen input, which already works.
- Sending the pen bytes nobody has observed. Only values captured from OnePlus Notes on the owner's own device are written to the pen.

The legal framing (EU Software Directive interoperability exceptions, as implemented in Dutch law) is in [`stylo2/docs/LEGAL.md`](stylo2/docs/LEGAL.md). It is general context, not legal advice.

## The database

[`research/`](research) holds the evidence base as plain JSON, one file per record type: sources, devices, interfaces, concepts, plans and open questions (gaps). Every record carries a status, the source it rests on, and the date it was checked:

| Status | Meaning |
| --- | --- |
| `verified` | Checked against a primary source: official documentation, the code in this repository, or a read-out from a real device |
| `unverified` | Reported somewhere (community posts, vendor specs) but not yet checked |
| `inferred` | A conclusion drawn from verified facts; a working hypothesis |
| `open` | Unknown; a question with a planned experiment |

Nothing is "known to work" until it has been tested on a device. Standard-library Python tools check and build it:

```bash
python3 research/tools/validate.py     # schema and cross-reference check
python3 research/tools/build.py        # combined file + per-record documents in research/build/
```

The records feed a searchable page ([`research/site/`](research/site)). Vendor firmware is recorded by path, size and SHA-256 only, never stored.

## Status

- **Host-verified:** the Rust workspace builds and its 26 tests pass. The Android modules compile. The database validates. The read-out tool runs against a simulated device.
- **Not yet tested on a device:** every route. The next step is the on-device session described in [`PICKUP.md`](PICKUP.md).

## Repository map

```
README.md                 This file: basis and scope
PICKUP.md                 Playbook for the local session with the Pad 3 attached
CLAUDE.md                 Ground rules loaded automatically by Claude Code
.claude/skills/           Claude Code skills: /pad3-readout, /route-a-poc, /record-findings
research/                 The database, its schema and tools, write-ups, reference material
  tools/pad3_readout.py   Read-only ADB read-out of the tablet (firmware, driver, service, logs)
  tools/fwinfo.py         Inspects pulled cps8601_firmware.bin files
  tools/aidl_check.py     Compares the tablet's pen-service interface with the reconstruction
stylo2/                   Rust workspace: backend-neutral haptics API, backends, FFI
  docs/                   Route analysis A–J, mission, trust boundaries, evidence ledger, legal
android/
  stylo2-poc/             Route A proof-of-concept app
  stylus-ble-inspector/   Route B recon app: scan, GATT discovery, read/write/subscribe, export
  stylo2-bridge/          Kotlin bridge to the Rust library
app/                      Upstream privileged companion app (reference only; needs root)
```

All Android modules build from the repository root with one Gradle wrapper:

```bash
./gradlew :stylo2-poc:installDebug             # Route A PoC on the connected tablet
./gradlew :stylus-ble-inspector:installDebug   # Route B inspector
cd stylo2 && cargo test --workspace            # Rust API
```

## Continuing the research locally

Clone the repository on the computer the tablet is plugged into, start Claude Code in it, and say:

> Read `PICKUP.md` and follow it. Ask me before anything that changes the tablet.

`PICKUP.md` takes the session through the tablet read-out and firmware retrieval, the Route A run, and, with the owner's go-ahead, the Bluetooth capture for Route B. It also says where each result goes in the database. The same steps are available as Claude Code skills:

| Skill | What it does |
| --- | --- |
| `/pad3-readout` | Read-only tablet read-out: project number, firmware pull and hashes, modules, driver, pen service and `safe.IOT` protection level, handshake |
| `/route-a-poc` | Installs (after asking) and runs the Route A PoC, captures its log, and maps the outcome to a trust boundary |
| `/record-findings` | Writes results into `research/data/` and the docs with honest status labels, validates, and commits |

## Credits and licences

- Forked from [YeetingWaterbottle/stylo2-haptics](https://github.com/YeetingWaterbottle/stylo2-haptics) by Devin Wang. The `app/` module and the reproduced `ISdkAidlInterface` come from there.
- `android/stylus-ble-inspector` reuses code from [BlueShark](https://github.com/nphil/BlueShark) (MIT, by nphil). See its [`NOTICE`](android/stylus-ble-inspector/NOTICE).
- `research/reference/fix-odm-oplus-pad2pro/` is an unmodified copy of jjhitel's GPL-3.0 module scripts, kept for analysis only.
- **Licence: not settled.** The Rust workspace's Cargo manifests declare `Apache-2.0 OR MIT`, but the repository has no `LICENSE` files, and the rest of it declares no licence. Choosing one, and checking the upstream terms for `app/`, is the owner's decision.
