# Working in this repository

Open Stylo 2 haptics research. The scope is in `README.md`, and the on-device playbook is in `PICKUP.md`. Read both before working with the tablet. The playbook's steps are also available as the skills `pad3-readout`, `route-a-poc` and `record-findings` in `.claude/skills/`.

## Rules that always apply

- **No cracking.** Don't bypass signature, permission or app-authentication checks, or extract or reuse vendor signing keys, certificates or tokens. If a route needs any of that, record it as a finding and stop that route.
- **No patching** of OnePlus apps or firmware, ever.
- **Decompiling: one owner-approved exception (2 Nov 2026).** The owner allowed JADX on the copy of `com.oplus.ipemanager` pulled from their own tablet, only to check the reconstructed interface. The conditions:
  - Look only at the pen-SDK interface classes (`ISdkAidlInterface`, `ISDKAidlCallback`).
  - The APK and decompiled output stay in `research/local/`.
  - Record only the descriptor, method names, transaction codes and signatures, through `research/tools/aidl_check.py`. Never copy code.
  - The process is PICKUP.md Step 2b. The legal basis is EU Software Directive Art. 6 (see `stylo2/docs/LEGAL.md`).
  - Nothing else gets decompiled: no other OnePlus app, no mirror download (such as APKMirror), and nothing beyond those classes.
  - Reading an APK's public signer fingerprint (`apksigner verify --print-certs`) still needs the owner's go-ahead.
- **The tablet is read-only.** Only run reads (`getprop`, `ls`, `cat`, `dumpsys`, `pm path`/`pm list`, `logcat -d`, `dmesg`, `modinfo`, `sha256sum`, `find`, `adb pull`, including pulling an installed APK). Use `research/tools/pad3_readout.py` where it covers the step. It enforces this.
- **Ask the owner first** before anything that changes the tablet: installing or uninstalling an app (including this repo's own apps), changing a setting (including the Bluetooth HCI snoop log), starting Shizuku, or writing to the pen.
- **Root:** never needed for what this project ships. If the tablet is rooted, `su` may be used for reads only.
- **Pen writes:** only bytes captured from OnePlus Notes on this device. No fuzzing.
- **Never report an unauthorised or failed call as success.** "Works" means the pen physically vibrated.
- **Pen input and OnePlus Notes must keep working.** If probing disrupts them, stop and tell the owner.

## Data

- Pulled files (firmware, kernel modules, APKs, full getprop dumps, logs, snoop captures) go in `research/local/`, which git ignores. Never commit them.
- Commit only hashes, sizes, short excerpts and findings. Redact Bluetooth addresses, serial numbers, IMEIs and account names as `<redacted>`.
- Results go in `research/data/*.json`. Set `status` honestly (`verified` only for what was read or observed on the device, with the build and date in `basis`), and set `updated` on every record you change. Route results also go in the `stylo2/docs/E-prototype.md` matrix.
- Run `python3 research/tools/validate.py` before committing. Use `py -3` instead of `python3` on Windows.

## Code

- Android modules build from the root: `./gradlew :stylo2-poc:assembleDebug`, `:stylus-ble-inspector:assembleDebug`, `:stylo2-bridge:assembleDebug`, `:app:assembleDebug`.
- Rust: `cd stylo2 && cargo test --workspace`. The public API shouldn't change. Findings fill the backend seams (`Stylo2GattProfile`, the transports, the `wire` mapping).
- `app/` is upstream code (the privileged route). Leave it unchanged unless the owner asks.
- Don't change AIDL transaction codes, interface descriptors or profile integers without device evidence.

## Git

Work on a feature branch and keep PRs as drafts until the owner reviews them. Don't force-push shared history.
