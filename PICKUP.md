# PICKUP — on-device testing handoff (OnePlus Stylo 2 / Pad 3)

**What this is:** a ready-to-use briefing for a coding agent continuing this
project on a local machine that has **ADB access to the owner's OnePlus Pad 3 +
Stylo 2**. Paste it as the agent's opening prompt, or point the agent at this
file. It is intentionally directive and self-contained; the detailed procedure
lives in [`stylo2/docs/G-local-continuation.md`](stylo2/docs/G-local-continuation.md).

---

## Mission

Find out how the Stylo 2's haptic feedback is actually driven and connected,
then light up whichever **custom-implementation route the device genuinely
permits** behind the existing Rust API — or establish, with evidence, that no
route is available to an ordinary non-root app and say so plainly. This is
interoperability work on hardware the owner owns; it is not about defeating
anything.

**Scope — narrow by design, and the asymmetry is a clue.** On this device
*everything* the stylus does already works **except** haptics in third-party
apps: detection, pressure, tilt, hover, normal input, the Bluetooth bond — and
haptics themselves inside OnePlus Notes. Pen *input* is an uplink (pen→tablet)
carried by Android's standard stylus/HID stack, so it is open to every app;
*haptics* is a downlink **control** command (tablet→pen) that OnePlus gates
behind its accessory service. So the actuator, the command path, and the link
are all proven good — the only missing piece is a third-party app's
**authorization to issue the control command**. Treat this as an authorization
wall on a working capability, not a broken transport or an unpaired device, and
**do not touch or reimplement pen input** (it already works).

## Where things stand — read before doing anything

- The repo already contains the **implementation scaffold**, host-verified only:
  - `stylo2/` — Rust workspace (`cargo test --workspace` → 26 passing; `clippy`
    clean). Backend-agnostic core + honest backends (`oplus`, `ble`, `android`)
    that report *unavailable/unauthorized* until a real transport is attached.
  - `android/stylo2-bridge/` — Kotlin integration layer (source only).
  - `android/stylus-ble-inspector/` — a **non-root BLE inspector APK** (scan →
    connect → discover GATT → read/write/subscribe → export). Source only;
    not yet built or run.
- **Nothing has been tested on a device.** Every "verified" claim so far is
  host-only. Your job is the device half.
- Open work lives on branch **`ccr-9fb81d0a-059pa6`** (PR #1, draft).
- Canonical brief: `stylo2/docs/MISSION.md`. Then `G-local-continuation.md`
  (step-by-step), `H-trust-boundaries.md` (the four-step experiment that
  isolates which boundary blocks), `J-capture-plan.md` (the nine captures +
  recording schema + tables), `I-evidence-ledger.md` (status + open
  questions→experiments), with `B-feasibility.md` / `C-protocol.md` /
  `E-prototype.md` for context and the test matrix.

## The one fact that shapes the whole approach

Haptics work today only through **OnePlus Notes**, which drives the accessory
service `com.oplus.ipemanager` over the **system's** Bluetooth link.

- To see **what Notes sends** (the real bytes) you need an **HCI snoop log**
  (BlueShark + Shizuku, no root). A second app's GATT connection **cannot** see
  Notes' traffic.
- The **BLE inspector** answers a different question: can *your own* app connect
  and write, and does that disrupt Notes?
- Do not assume the control path is plain GATT — the service is `.btadsorb.*`.
  **The snoop decides** whether it's GATT, another channel, or service-brokered.

Use Notes as the known-good oracle; observe, don't modify it. The Pad 3's
split-screen lets you keep Notes drawing-active while an observer pane captures —
see `docs/G` §5.

## Hard boundaries — non-negotiable, do not "work around"

- **No root, no Magisk, no `/system/priv-app` install, no firmware changes.**
  This project excludes the privileged route by design. If a route needs it,
  that is a finding to report, not a thing to do.
- **Never present an unauthorized or failed call as success.** If a route needs
  `com.oplus.permission.safe.IOT` and the app can't hold it, report
  `PermissionDenied` and stop that route.
- **Interoperability only:** observe the owner's own device and its own app's
  traffic, then independently reimplement. Do **not** defeat protections or
  extract/reuse vendor signing keys, certificates, or tokens. See
  `stylo2/docs/LEGAL.md`.
- **BLE writes change hardware state:** only ever write bytes you **captured
  from Notes** (known-safe). No blind fuzzing of the pen.
- **Keep the device working:** normal stylus input and OnePlus Notes must keep
  functioning. Non-interference is a required test (`docs/E` 11–12), not
  optional — if your probing disrupts the system link, stop and report.

## First actions (ordered — fastest verifiable route first)

1. Confirm the host baseline: `cd stylo2 && cargo test --workspace` (expect 26).
2. **Route A — the shortest path to a working PoC.** Build and install the
   minimal PoC on the pad: `cd android/stylo2-poc && ./gradlew :app:installDebug`.
   Open it, tap **Bind**, pick a profile, tap **Trigger**, and **watch the
   pen**. If it vibrates, **you are done** — that app is the PoC. Its
   [`README.md`](android/stylo2-poc/README.md) maps every outcome to the
   boundary it implicates. (Expect to fix real Android compile errors first —
   the module was authored without an SDK to build against; keep fixes minimal.)
3. If Route A is refused: **device inventory** (`docs/G` §3) — especially the
   **protection level of `com.oplus.permission.safe.IOT`** — and the four-step
   isolating experiment in `docs/H-trust-boundaries.md` to confirm which
   boundary blocks (the PoC already stages this; mostly cross-check).
4. **Route B — direct communication.** One HCI snoop capture of Notes
   (`docs/G` §4) to learn whether haptics are even GATT and what the bytes are;
   then the nine-scenario protocol in `docs/J-capture-plan.md`. Use the BLE
   inspector (`android/stylus-ble-inspector`, `docs/G` §6) for independent-access
   and non-interference tests.
5. Record every result with a method tag (inspection / static / adb / physical)
   in `docs/E-prototype.md`, and promote confirmed claims in
   `docs/I-evidence-ledger.md`.

## Three possible outcomes — let the evidence pick, don't force one

1. **Direct BLE works** — the snoop shows a writable characteristic, your app
   can write a captured value and the pen buzzes, and Notes still works with the
   inspector connected → fill `stylo2_ble::Stylo2GattProfile` (UUIDs + byte
   encoder), write a Kotlin `GattTransport`, and set
   `Availability.ble_protocol_verified`.
2. **Only the privileged binder path** — control is reachable only via the
   accessory service (needs `safe.IOT`, ungrantable to an ordinary app) and no
   independent GATT path works → report that the only legitimate route is
   **official OnePlus developer authorisation**. Do not root.
3. **In-between** — e.g. GATT present but encrypted/authenticated, or a second
   connection kills the system link → document precisely in `docs/C`.

## Feed results back into the code

- **BLE:** `stylo2/crates/stylo2-ble` — populate `Stylo2GattProfile`; use
  `stylo2_ble::recon::candidate_control_characteristics` to rank candidates from
  the inspector's exported GATT table.
- **Binder (only if authorised):** implement the Kotlin `BinderTransport` over
  the existing `StylusVibrationClient`; verify the profile integers empirically
  (`setVibrationType(0..4)` + feel the pen) and correct
  `stylo2-oplus::wire::reported_code`.
- **Docs:** promote confirmed items in `stylo2/docs/C-protocol.md` from
  *reported/hypothetical* → *verified*; fill the `docs/E` matrix.

## Working conventions

- Work on `ccr-9fb81d0a-059pa6` (PR #1), or a branch off it; follow the repo's
  existing commit/PR conventions. Do not force-push shared history.
- The Rust **public API should not need to change** — only the backend seams
  (`Stylo2GattProfile`, `BinderTransport`/`GattTransport`, the `wire` mapping)
  get filled in. Keep changes scoped to what a finding requires.
- Commit device findings as doc updates (`docs/C`, `docs/E`) alongside any code.

## Stop and ask the owner when

- A choice is ambiguous or architecturally significant.
- A route needs access you don't have (a permission, vendor authorisation) —
  report it, don't improvise around it.
- Probing disrupts the system link or Notes — stop before continuing.
- You would need to do anything on the "never" list above to make progress.
