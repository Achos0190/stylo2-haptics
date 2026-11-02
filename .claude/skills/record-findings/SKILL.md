---
name: record-findings
description: Write results from the Pad 3 (read-out, PoC run, handshake, Bluetooth captures) into the research database and docs, with honest status labels, then validate and commit. Use after any on-device step, or when the owner asks to record, save or commit findings.
---

# Record findings

The database is `research/data/*.json`, one file per record type. Its rules are in `research/README.md` and the schema is `research/schema/record.schema.json`. Read both before editing if they aren't already in context.

## Sources of truth

- `research/local/pad3/*.json` and `summary.md`, from `pad3_readout.py`. These are redacted.
- What the owner observed physically, such as "the pen vibrated" or "no vibration".
- Never anything you inferred without saying so.

## Where things go

| Finding | Record or file |
| --- | --- |
| Project number, model, build, root and fix-module state, firmware folders | `devices.json` → `dev-oneplus-pad3` |
| Firmware builds, hashes, which folder holds which | `interfaces.json` → `if-cps8601-firmware`, plus `research/reference/pad3-firmware/MANIFEST.md` (paths, sizes, hashes; no binaries, same style as `research/reference/oplus-global-pen-enabler/MANIFEST.md`) |
| Driver module info, firmware-path format | `interfaces.json` → `if-oplus-wireless-pen-driver` |
| Pen service APK version, size, SHA-256; interface-check result (`research/local/pad3/aidl-check.json`) | `interfaces.json` → `if-oplus-ipemanager`, `if-oplus-sdk-interface`; `gaps.json` → `gap-aidl-matches-firmware`. Names, codes and signatures only, never decompiled code |
| Pen service version, confirmed names, `safe.IOT` protection level | `interfaces.json` → `if-oplus-ipemanager` |
| Answers to open questions | the matching `gap-*` in `gaps.json` (the order is in `PICKUP.md`) |
| The read-out itself | `sources.json` → `src-pad3-readout` (what was read and when; no identifiers) |
| Handshake sequence, Pad 3 row, firmware table | `research/docs/pen-attach-handshake.md` |
| Route test results | `stylo2/docs/E-prototype.md` matrix (method tag: inspection / static / adb / physical) and `stylo2/docs/C-protocol.md` |

## Status rules

- `verified`: read or observed on this tablet. The `basis` says how, for example "Read from the Pad 3 (OPD2415, build <ro.build.display.id>) with pad3_readout.py on <date>".
- `inferred`: a conclusion from verified facts. Say which facts in `basis`.
- Close a `gap-*` (change its status from `open`) only with direct evidence. Otherwise narrow it in `details` and keep it `open`.
- Set `updated` to today on every record you change, and keep `related` IDs valid.
- Redact Bluetooth addresses, serial numbers, IMEIs and account names as `<redacted>`.

## Finish

```bash
python3 research/tools/validate.py && python3 research/tools/build.py
git status    # nothing under research/local/ or research/build/ may be staged
git add research stylo2/docs
git commit -m "Record Pad 3 <what>: <one-line result>"
```

Push only if the owner wants it. If the session has the Artifact tools, sync the changed records to the database page as described in `research/README.md`. Otherwise list the changed record IDs for the owner.
