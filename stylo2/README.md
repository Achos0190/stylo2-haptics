# stylo2 — independent haptic integration (Rust workspace)

A Rust-first, reusable library that lets a third-party app drive the **OnePlus
Stylo 2** stylus's advanced haptic feedback, with a minimal Kotlin/Android
bridge. It targets a **stock, non-rooted OnePlus Pad 3** (OxygenOS, Android 16).

This workspace is the independent, interoperability-focused reworking requested
in the project specification. It is **separate from** the existing
root/Magisk-based app at the repository root: nothing here requires root, a
privileged (`/system/priv-app`) install, or firmware modification. Where the
device does not grant an ordinary app the access a route needs, the library
says so honestly instead of pretending to work.

> **Continuing on a device?** A coding agent should start from
> [`../PICKUP.md`](../PICKUP.md) (handoff briefing) and follow
> [`docs/G-local-continuation.md`](docs/G-local-continuation.md) (step-by-step).

## Status at a glance

| Thing | Status |
| --- | --- |
| `stylo2-core` portable model + policy | **Implemented, compiles, 22 tests pass on host** |
| Backend scaffolds (`oplus`, `ble`, `android`) | **Implemented as honest adapters** (inert/"unavailable" until a device transport + verified protocol are attached) |
| `stylo2-ffi` C ABI + JNI entry points | **Implemented**; C ABI tested on host, JNI module type-checks against `jni 0.21` |
| Kotlin bridge | Source provided (`../android/stylo2-bridge`), not built here (needs Android SDK) |
| A working haptic command **on a physical pen** | **Not demonstrated** — requires the device and a proven access route (see `docs/B-feasibility.md`) |

Everything marked "compiles/tests" was verified on an `x86_64-linux` host. **No
physical-device, emulator, or ADB testing has been performed in this
environment.** Claims that need the OnePlus Pad 3 are called out as such.

## Why a route may not exist on stock firmware

The pen's haptic motor is driven by OnePlus's accessory service
(`com.oplus.ipemanager`). Reaching it over binder requires the restricted
permission `com.oplus.permission.safe.IOT`, which a stock device does not grant
an ordinary third-party app. The existing root app sidesteps that by installing
itself as a privileged system app — which is exactly the mechanism this project
excludes. The legitimate, non-root routes and their (currently unverified)
feasibility are analysed in [`docs/B-feasibility.md`](docs/B-feasibility.md).

## Layout

```
stylo2/
├── Cargo.toml                     workspace
├── crates/
│   ├── stylo2-core/               portable API, states, profiles, events, caps,
│   │                              errors, Backend trait, HapticManager policy
│   ├── stylo2-oplus/              OnePlus accessory-service (binder) adapter
│   ├── stylo2-ble/                direct-BLE transport scaffold (inert until verified) + recon ranker
│   ├── stylo2-android/            permission/service names, backend-selection, fallback
│   └── stylo2-ffi/                C ABI + optional JNI entry points
└── docs/                          deliverables A–F and legal/interoperability notes
```

The Android side lives under [`../android`](../android): the Kotlin bridge
(`stylo2-bridge`), the on-device BLE reconnaissance app (`stylus-ble-inspector`),
and the minimal **Route-A haptic PoC** (`stylo2-poc`) — the fastest path to a
verifiable physical result, which doubles as the trust-boundary isolating
experiment.

## Build & test

```bash
cd stylo2
cargo test --workspace                                   # 22 tests, no device needed
cargo run -p stylo2-core --example demo --features mock  # end-to-end demo on the mock backend
```

Android/JNI build (needs the Android NDK + `cargo-ndk`, not done here):

```bash
cargo ndk -t arm64-v8a build -p stylo2-ffi --release --features jni
```

## Deliverables

See [`docs/`](docs/):

- **Mission** — [`MISSION.md`](docs/MISSION.md): the canonical full-stack investigation brief, the deliverable map, and the (deliberate, evidence-driven) crate/state-layout decisions.
- **A** — [`A-reconnaissance.md`](docs/A-reconnaissance.md): interfaces, APKs, AIDL, permissions, BLE, SDKs, sources, each classified *verified / reported / hypothetical*.
- **B** — [`B-feasibility.md`](docs/B-feasibility.md): the four routes, feasibility, the preferred investigation order.
- **C** — [`C-protocol.md`](docs/C-protocol.md): the reported AIDL/commands/profiles and the BLE unknowns, with an on-device recon procedure.
- **D** — [`D-implementation-plan.md`](docs/D-implementation-plan.md): crate responsibilities, API, Kotlin boundary, permissions, concurrency, tests, distribution.
- **E** — [`E-prototype.md`](docs/E-prototype.md): what the prototype demonstrates, and the physical-device test matrix.
- **F** — [`F-crate.md`](docs/F-crate.md): integration and reuse.
- **G** — [`G-local-continuation.md`](docs/G-local-continuation.md): how to continue on a local machine with the Pad 3 over ADB — device inventory, HCI-snoop capture of OnePlus Notes, the Notes + observer split-screen tandem method, independent GATT + non-interference testing, and feeding results back into the code.
- **H** — [`H-trust-boundaries.md`](docs/H-trust-boundaries.md): architecture diagram, the trust-boundary/enforcement-point table, the four-step experiment that isolates *which* boundary blocks third-party haptics, and the target state machine.
- **I** — [`I-evidence-ledger.md`](docs/I-evidence-ledger.md): every claim with status (verified/reported/hypothesis), source, and how to upgrade it — plus open questions mapped to the exact experiment for each.
- **J** — [`J-capture-plan.md`](docs/J-capture-plan.md): the nine controlled packet captures, the per-experiment recording schema, empty correlation tables, and how the results flow into the code.
- **Legal** — [`LEGAL.md`](docs/LEGAL.md): EU/Dutch interoperability framing (not legal advice).

## License

Dual-licensed under Apache-2.0 OR MIT.
