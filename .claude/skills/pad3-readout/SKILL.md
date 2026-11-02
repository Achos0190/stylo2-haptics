---
name: pad3-readout
description: Read-only read-out of the connected OnePlus Pad 3 for the Stylo 2 research. Covers the project number, pen-charger firmware retrieval and hashing, root modules, the wireless-pen driver, the pen service and its permission, pulling the pen service APK, the owner-approved interface check, the Bluetooth bond, and the pen attach handshake. Use when the owner asks to check, read out or inspect the tablet, to pull the cps8601 firmware or the pen service APK, to check the AIDL interface, or to run PICKUP.md Steps 1, 2, 2b or 4.
---

# Pad 3 read-out

Runs `research/tools/pad3_readout.py`, which only reads from the tablet. It refuses any command that isn't on its read-only allowlist, and it redacts serial numbers, Bluetooth addresses, IMEIs and email addresses in everything except `research/local/pad3/raw/`.

Use `py -3` instead of `python3` on Windows. If the owner names steps in the arguments (`$ARGUMENTS`), run only those.

## 1. Check

```bash
python3 research/tools/pad3_readout.py check
```

- The device state must be `device`. If it shows `unauthorized` or nothing, ask the owner to accept the USB-debugging prompt, then stop until they have.
- If the tablet shows a root prompt, the owner may approve or deny it. Either is fine: root-only reads are skipped without it.
- Tell the owner what `check` found: model, `ro.boot.prjname`, build, boot state, root yes/no, and which apps are installed.

## 2. Read-out (non-interactive)

```bash
python3 research/tools/pad3_readout.py all
```

Then read `research/local/pad3/summary.md`. It is redacted and safe to quote. For any firmware build marked `unknown build`, and for any two distinct builds:

```bash
python3 research/tools/fwinfo.py --strings research/local/pad3/raw/firmware/wireless_pen/<folder>/cps8601_firmware.bin
python3 research/tools/fwinfo.py --diff <a.bin> <b.bin>
```

Point out to the owner:

- whether the `ro.boot.prjname` folder exists, and which build each folder holds (compare against the known builds `cfac6737…` and `33a4ab21…`);
- whether a `24976` build exists and how it differs from `24926`;
- any fix module or overlay;
- the driver `vermagic` and any `wireless_pen/%s/...` path string;
- **the `safe.IOT` protection level.** If it contains `signature`, an ordinary app can never hold it, so Route A should fail at bind;
- which packages hold `safe.IOT`;
- the pulled `com.oplus.ipemanager` version and SHA-256 (`apk` step), next to the APKMirror 15.0.30 listing the owner pointed to (`src-apkmirror-smart-accessories`).

## 3. Interface check (owner-approved; PICKUP.md Step 2b)

Follow PICKUP.md Step 2b exactly:

- JADX runs only on the APK pulled in step 2, and only on the `ISdkAidlInterface` and `ISDKAidlCallback` classes.
- Ask the owner before any full decompile.
- Run `python3 research/tools/aidl_check.py <jadx-output> --apk-version <versionName>` and report its table.
- Never paste decompiled code into chat, commits or records; only names, codes and signatures.
- Don't change the app's interface on a difference. Report it to the owner.

## 4. Handshake (only when the owner is ready to handle the pen)

```bash
python3 research/tools/pad3_readout.py handshake
```

The tool asks the owner to detach and re-attach the pen, then press Enter. Compare the markers with `Rx attach -> tx wakeup -> receive ask pkt -> get ble addr -> report 1` from `research/docs/pen-attach-handshake.md`.

## Rules

- Don't run raw `adb shell` commands that the tool already covers. If you need a read it doesn't do, use only the reads listed in `CLAUDE.md`, and ask before anything else.
- Never commit anything under `research/local/`. Quote only from `summary.md` and the step JSON files.
- To write results into the database, use the `record-findings` skill.
