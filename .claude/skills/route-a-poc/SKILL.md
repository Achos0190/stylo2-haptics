---
name: route-a-poc
description: Run the Route A proof-of-concept app (android/stylo2-poc) on the connected Pad 3 and work out which trust boundary, if any, stops an ordinary app from triggering Stylo 2 haptics. Use when the owner wants to test the designed app, try the PoC, or run PICKUP.md Step 3.
---

# Route A: unprivileged PoC run

The PoC binds `com.oplus.ipemanager` (action `com.oplus.ipemanager.ACTION.PENCIL_SDK`) as an ordinary app, with no root and no privileged install. It logs each stage separately: ENV, DISCOVERY, BIND, AUTHZ and EXEC. Every on-screen line is also sent to logcat under the tag `Stylo2Poc`.

## 1. Before installing

- Run the `pad3-readout` skill's `service` step first if `research/local/pad3/service.json` doesn't exist yet. The `safe.IOT` protection level predicts the outcome.
- **Ask the owner** before installing: it is a change to the tablet. Install it as an ordinary app only. Never push it to `/system` or use Magisk.

```bash
./gradlew :stylo2-poc:installDebug        # Windows: .\gradlew :stylo2-poc:installDebug
```

## 2. The run (the owner does this on the tablet)

1. Draw in OnePlus Notes once, to confirm the pen works.
2. Open **Stylo2 Haptics PoC**.
3. Tap **1. Bind / probe service**.
4. Pick a profile and tap **2. Trigger haptic**, while watching the pen.
5. Tap **3. Stop feedback**.

## 3. Capture and interpret

```bash
python3 research/tools/pad3_readout.py poc-log
```

The tool asks whether the pen physically vibrated. Record exactly what the owner says.

| Log shows | Boundary | Next |
| --- | --- | --- |
| BIND and AUTHZ pass, and **the pen vibrates** | none: Route A works unprivileged | Map the profiles (`setVibrationType` 0–4, and how each feels), then the Kotlin `BinderTransport` |
| `SecurityException` or `bindService returned false` at BIND | B1: permission enforced | Route B (`stylo2/docs/G-local-continuation.md` §4–6) |
| BIND passes, AUTHZ throws | B2: the service checks the caller | Route B, or official OnePlus SDK authorisation |
| Calls return, no vibration | B4: session or protocol gates the motor | Route B snoop |
| Not installed, or no discovery match | service missing on this build | Re-check the read-out |

The boundaries are explained in `stylo2/docs/H-trust-boundaries.md`.

## Rules

- "Works" only if the pen physically vibrated. A call that returns without vibration is not success.
- If BIND or AUTHZ is refused, that is the finding. Don't try to escalate privileges, grant the permission, spoof the package or signature, or use the upstream privileged app.
- Don't change AIDL transaction codes, the interface descriptor or the profile integers to "make it work". A descriptor mismatch is itself evidence to record.
- Write the result into the `stylo2/docs/E-prototype.md` matrix and the database with the `record-findings` skill.
