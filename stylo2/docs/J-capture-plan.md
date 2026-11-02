# Deliverable J — Packet-capture analysis plan

The controlled experiment that turns "how does Notes drive the pen" into
reproducible, action↔traffic correlations. This is the method and the empty
tables; fill them on the Pad 3. It operationalises `G-local-continuation.md`
§4–5 and feeds the evidence ledger (`I`) and the backend seams.

## Principles for this capture

- **OnePlus Notes is the oracle.** It is the only known-good haptic driver;
  observe it, do not modify it.
- **HCI snoop first.** It is below the stack and sees the *system's* link to the
  pen — which a second app's GATT client cannot. Use BlueShark+Shizuku (no root)
  or Developer-Options snoop + `adb bugreport` (G §4).
- **An external BLE sniffer** (e.g. nRF52840 + Wireshark) only adds value if the
  HCI snoop is missing packets (e.g. a channel the host controller doesn't log);
  and if the link uses LE encryption, an over-the-air sniffer sees ciphertext
  unless it captured the pairing — so the **HCI snoop is strictly more
  informative** here. Note this limitation; don't reach for a sniffer first.
- **Absence of an observed packet does not prove absence of a path** (mission
  6). Record what you *did not* see too.
- Change **one variable per capture**; keep Notes' tool/canvas identical between
  runs so diffs are meaningful.

## The nine captures

Run each as its own snoop segment (or one long capture with named markers at
each step — BlueShark markers are ideal).

| # | Scenario | Exact action | What to look for |
| --- | --- | --- | --- |
| 1 | Idle baseline | Pen connected, Notes open, **no** drawing for ~15 s | background/keepalive traffic to subtract from the rest |
| 2 | Pencil profile | Select Pencil in Notes | the write(s) that change vs idle |
| 3 | Eraser profile | Select Eraser | which byte(s) differ from #2 |
| 4 | Ballpoint profile | Select Ballpoint | which byte(s) differ from #2/#3 |
| 5 | Pen profile | Select Pen | which byte(s) differ |
| 6 | Feedback ON | Draw so haptics fire (vibration felt) | the activation write; note handle + bytes |
| 7 | Feedback OFF | Stop drawing / lift / toggle off | the deactivation write |
| 8 | Disconnect / reconnect | Detach+reattach or toggle pen BT | teardown + re-init sequence |
| 9 | Initial connection / session | From powered-off pen: connect fresh with snoop already running | any **handshake/session init** before the first haptic (B4) |

Capture #9 is the one that reveals whether a session/auth step gates haptics.

## Per-experiment recording schema (mission §6)

For every captured action, record:

```
scenario:            <1–9 / name>
action:              <exact user action>
timestamp:           <wall clock, and marker id if used>
app/process:         <foreground app; binder caller if observable via dumpsys/logcat>
binder calls:        <method(s) seen, if observable>
bt_state:            <connected/for how long; address; role>
packet.direction:    <host→pen / pen→host>
packet.opcode:       <ATT opcode or other, e.g. Write Request / Write Command / Handle Value Notification>
packet.handle:       <ATT handle>
packet.uuid:         <characteristic UUID if resolvable from this capture's discovery>
packet.payload:      <hex bytes>
error:               <any error response>
vibration_observed:  <yes/no — did the pen physically buzz?>
```

The `vibration_observed` field is the ground truth that anchors every
correlation; log it honestly per step.

## Correlation tables (fill on device)

### Profile selection → write (scenarios 2–5)

| Profile | handle | char UUID | payload (hex) | differs from Pencil at byte(s) | vibration? |
| --- | --- | --- | --- | --- | --- |
| Pencil | | | | — | |
| Eraser | | | | | |
| Ballpoint | | | | | |
| Pen | | | | | |

### Feedback on/off (scenarios 6–7)

| Action | handle | char UUID | opcode | payload (hex) | vibration? |
| --- | --- | --- | --- | --- | --- |
| Start | | | | | |
| Stop | | | | | |

### Session / connection (scenarios 8–9)

| Step | direction | opcode | handle | payload (hex) | notes (handshake? subscribe CCCD? MTU/PHY?) |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

## From tables → code

- **If scenarios 2–7 show ATT writes to one characteristic that correlate with
  vibration:** that characteristic is the control attribute. Populate
  `stylo2_ble::Stylo2GattProfile { service, control_characteristic, encode }`,
  where `encode` reproduces the exact per-command payloads above. Rank-check
  with `stylo2_ble::recon::candidate_control_characteristics` against the
  inspector's discovery export.
- **The per-profile bytes** verify or correct `stylo2-oplus::wire::reported_code`
  (they must match whichever path — binder `setVibrationType(int)` arg, or the
  BLE payload — you are implementing).
- **If scenario 9 shows a handshake** before any haptic, that is B4 session init:
  model it as the `Initialising` state (see `H-trust-boundaries.md`) and
  reproduce it before commands.
- **If no haptic-correlated ATT traffic appears at all**, haptics are not a
  plain GATT write from the host — record that (it rules the direct-BLE route
  out or points to another channel), and the binder/official-SDK route becomes
  the focus.

Promote every confirmed row into `I-evidence-ledger.md` (→ [V-device]) and the
`E-prototype.md` matrix with the `physical` method tag.
