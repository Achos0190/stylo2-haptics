# Stylus attach handshake and the regional-ROM stylus fix

How a OnePlus tablet first recognises a Stylo pen, why that breaks after converting a Chinese ColorOS tablet to a global OxygenOS ROM, and what the two community fixes change.

This is not the haptics path. It matters for haptics because nothing works on the pen until this handshake succeeds: no pairing, no pressure over Bluetooth, no vibration.

Status labels follow the research database: **verified** (checked against the source named), **inferred** (reasoned from verified facts), **unverified** (reported, not checked), **open** (unknown).

## Summary

- The tablet has a wireless-charging chip for the pen, the **CPS8601**, driven by the kernel module `oplus_wireless_pen.ko` ("Wireless Pen Charger Cps8601", "QTI Glink wireless pen driver"). *Verified (module info quoted in the cxzlw repo).*
- When the pen snaps on, the chip detects it and receives packets from the pen over the charging link. One packet carries the **pen's Bluetooth LE address**, which the driver reports onward. *Verified (kernel log in the cxzlw repo).*
- The chip runs firmware loaded from `/odm/firmware/wireless_pen/<prjname>/cps8601_firmware.bin`. `<prjname>` is the tablet's hardware project number, read from the `ro.boot.prjname` system property. *Strongly supported: jjhitel's fix creates exactly that directory, and users report it works.*
- A global ROM installed on Chinese hardware (or the reverse) ships firmware only under its own project numbers. The tablet then looks for a directory that isn't there, the handshake never completes, and the pen never pairs.
- Both fixes put a copy of existing firmware where the tablet looks. Neither ships new or modified firmware logic.

## Devices and project numbers

| Hardware | Codename | Project numbers seen | Source |
| --- | --- | --- | --- |
| OnePlus Pad Pro (China) / OnePlus Pad 2 (global) | caihong / rainbow | `23926`, `23927`, `23976`, `23978` | cxzlw module; crDroid caihong tree. *Verified* |
| OnePlus Pad 2 Pro (China) | — | `24926`, `24976` | jjhitel module v1.0 accepted only these two. *Verified* |
| OnePlus Pad 3 (global, OPD2415), converted from Chinese hardware (OPD2413) | — | Probably `24926` or `24976` | Appears to be the same hardware family as the Pad 2 Pro. *Inferred; confirm with `getprop ro.boot.prjname`* |
| OnePlus Stylo 2 (Chinese name: OnePlus Smart Stylus Pro) | — | — | The pen. *Verified* |

The leading digits look like the project year: `239xx` for the 2024 tablets designed in 2023, `249xx` for the 2025 tablets. *Inferred*

Which number in each pair is Chinese and which global is not established. jjhitel's v1.0 copied `24976`'s firmware into `24926`, so on that author's converted tablet the hardware reported `24926` and the installed ROM shipped `24976`. *Verified (script); region mapping inferred*

## Components

### Kernel driver

```
filename:     /vendor/lib/modules/oplus_wireless_pen.ko
description:  Wireless Pen Charger Cps8601
description:  QTI Glink wireless pen driver
alias:        i2c:cps-wls-charger
depends:      panel_event_notifier,oplus_bsp_boot_projectinfo,pmic_glink
vermagic:     6.1.68-android14-11-o-g2ba5638f614f SMP preempt mod_unload modversions aarch64
license:      GPL v2
```

- `i2c:cps-wls-charger`: the chip sits on an I²C bus.
- `pmic_glink`: the driver talks through Qualcomm's GLINK link to the power-management firmware.
- `oplus_bsp_boot_projectinfo`: Oplus's project-information module, which identifies the hardware variant. This fits the per-project firmware directories.
- `panel_event_notifier`: the driver reacts to display on/off events. *Inferred from the name*
- This module info is from the Pad Pro / Pad 2 kernel (`6.1.68-android14`). The Pad 3 may ship a different driver version.
- The GPL v2 licence means OnePlus should publish the driver source in its kernel releases. That source would show exactly how the directory is chosen. *Open*

### Chip

- Named CPS8601 in the driver. The `cps` prefix suggests ConvenientPower Systems, a wireless-power chip maker. *Unverified*
- The cxzlw author describes it as a Qi 1.3 wireless charging chip. *Unverified*
- Its firmware strings show it acting as the **charging transmitter**: TX mode, Qi pings, foreign-object detection from power loss, coil calibration, and receiver-removal logging. *Verified (strings, see below)*

### Firmware files

From the cxzlw module (`odm/firmware/wireless_pen/`). The binaries are not stored in this repository; see [`reference/oplus-global-pen-enabler/MANIFEST.md`](../reference/oplus-global-pen-enabler/MANIFEST.md).

| Directory | Size | SHA-256 | Added |
| --- | --- | --- | --- |
| `23926` | 15,360 B | `cfac6737…400f9e81` | 6 Mar 2025 (original commit) |
| `23927` | 15,360 B | `cfac6737…400f9e81` | 6 Mar 2025 |
| `23976` | 15,360 B | `cfac6737…400f9e81` | 6 Mar 2025 |
| `23978` | 15,360 B | `cfac6737…400f9e81` | 6 Mar 2025 |
| `24926` | 15,360 B | `33a4ab21…6dd6aa1ee4` | 12 Apr 2026 ("Add new firmware to fix it", contributor FRTI64) |

- The four `239xx` copies are byte-identical. `24926` is a different build: 9,675 bytes differ, and it fills slightly more of the 15 KB image (15,065 bytes against 14,972). *Verified*
- The crDroid device tree for caihong lists real files in `23976` and `23978`, with `23926` and `23927` created as symlinks to them:

  ```
  odm/firmware/wireless_pen/23976/cps8601_firmware.bin;SYMLINK=odm/firmware/wireless_pen/23926/cps8601_firmware.bin
  odm/firmware/wireless_pen/23978/cps8601_firmware.bin;SYMLINK=odm/firmware/wireless_pen/23927/cps8601_firmware.bin
  ```

### What is inside the firmware

The image looks like code for an 8051-family microcontroller. *Inferred from the evidence below.*

- It opens with long-jump instructions (`02 xx xx`) at offsets `0x0000`, `0x0003` and `0x000B`. These are an 8051's reset, external-interrupt-0 and timer-0 vector slots.
- Its format strings use `%bu`, the byte-sized integer format of the Keil C51 compiler.

Readable strings, the same in both builds:

| Area | Strings |
| --- | --- |
| Transmit and ping | `cmd enter tx mode`, `Ping:Freq=%u,Duty=%u,VIN=%u,VPA=%u,IPA=%u`, `fre=%u;duty=%u;` |
| Power and foreign-object detection | `RP=%u;RPpower=%u;Papower=%u;Vpa=%u;Ipa=%u;Vpk=%u`, `ploss_rx=%d ploss_tx=%d`, `ploss=%d,ploss_th=%d`, `ploss fod trigger`, `dlp!maxpower %d`, `###ovpk %d %d` |
| Coil calibration | `q cali`, `save Q value successful`, `save Q value fail`, `cail cnt:%d cail width:%d …`, `cail_times %bu`, `adcq :` |
| State and control | `RX remove`, `low power`, `int_enable=%u, int_type=%u`, `clear int_l=%bu`, `clear int_h=%bu`, `start crc check`, `sys_reset` |
| Factory mode | `cmd enter equipent mode`, `exit equipment mode`, `enter idcfg` |
| Version | `ver%bu.%bu` (printed at run time, not stored as text) |

The chip firmware contains nothing about haptics, vibration or Bluetooth. It handles power transfer and the low-level packets the pen sends back over the charging link. *Verified (strings)*

## Attach handshake

The cxzlw author captured this kernel log while attaching and removing the pen on a Chinese ROM:

```
[cps8601]:irq_flag = 0x200
[cps8601]:Rx attach!
[cps8601]:irq_flag = 0x5
[cps8601]:tx wakeup!
[cps8601]:[error_attach_check_work_func] count 1
[cps8601]:irq_flag = 0x8
[cps8601]:irq_flag = 0x10
...
[cps8601]:[cps_wls_get_ask_packet]get add_check
[cps8601]:irq_flag = 0x20
[cps8601]:receive ask pkt!
[cps8601]:[cps_wls_get_ask_packet]get ble addr(956)
[cps8601]:cps8601_valid_check: .. .. ..
[cps8601]:[cps_wls_get_ask_packet]report 1
```

Read as a sequence (*inferred*):

1. The pen lands on the magnetic charger and the chip raises an attach interrupt (`Rx attach!`).
2. The chip wakes and starts transmitting power (`tx wakeup!`). The driver watches for a failed attach.
3. The pen talks back by modulating the charging load. Qi calls this ASK (amplitude-shift keying) communication; `receive ask pkt!` is one packet.
4. One packet carries the pen's Bluetooth LE address (`get ble addr`). The driver validates it (`cps8601_valid_check`) and reports it (`report 1`), presumably to the accessory service so it can pair over Bluetooth LE without user action.
5. From here, pen traffic, including haptic commands from `com.oplus.ipemanager`, goes over Bluetooth LE.

With no firmware in the expected directory, step 3 never produces a valid address and the pen never pairs.

## The two fixes

| | cxzlw `oplus-global-pen-enabler` | jjhitel `fix-odm-oplus-pad2pro` (XDA thread) |
| --- | --- | --- |
| Type | KernelSU module | Magisk module (installer needs Magisk 20.4+; the README also mentions other root managers) |
| Ships firmware | Yes: copies for `23926`, `23927`, `23976`, `23978`, `24926` | No: copies what the ROM already has |
| Picks the directory | Covers every number it knows | Reads `ro.boot.prjname` at install time |
| How files reach `/odm` | Module-root `odm/` folder; relies on the root manager to mount it | Its own overlay mount in `post-fs-data.sh` |
| Licence | None stated | GPL-3.0 |
| Latest change | 12 Apr 2026 (adds `24926`) | v1.2, 19 Aug 2025 |

A third route, flashing the matching region's whole `odm.img`, is covered under [Choosing a fix](#choosing-a-fix).

### jjhitel module v1.2, step by step

Source: the uploaded `fix-odm-oplus-pad2pro_v1.2.zip`, whose scripts are byte-identical to the GitHub repo at commit `0131a9d`. A reference copy is in [`reference/fix-odm-oplus-pad2pro/`](../reference/fix-odm-oplus-pad2pro/).

**Install (`customize.sh`)**

1. Reads `ro.boot.prjname` and `ro.product.system.manufacturer`. Aborts unless the project number is exactly five digits and the manufacturer is `oplus`.
2. Lists `/odm/firmware/wireless_pen/`, keeps five-digit names other than the tablet's own, and sorts them.
3. If there is one, it uses it. If there are several, it takes the first one without a marker file and creates the marker. The aim is for each re-flash to try the next directory.
4. Copies every `*.bin` from the chosen directory into the module's `odm/firmware/wireless_pen/<prjname>/`.
5. Sets owner `0:0`, folders `0755`, files `0644`, and SELinux context `u:object_r:vendor_configs_file:s0`.

**Every boot (`post-fs-data.sh`)**

1. Creates `/dev/anyfs/upper/odm/firmware` and a work folder. `/dev` is in RAM, so this is rebuilt on each boot.
2. Copies the module's `odm/firmware/*` into the upper folder.
3. Copies mode, owner and SELinux context from the real `/odm/firmware` onto everything it copied.
4. Mounts an overlay on `/odm/firmware`, with the stock folder underneath and the copied files on top.
5. The new `<prjname>` folder then appears alongside the stock ones. Nothing on the ODM partition is changed.

Because the module mounts itself, it does not depend on whether the root manager mounts a top-level `odm/` folder. The changelog says v1.1 added this "for abnormal cases" (credit AestasBritannia).

**Version history**

| Version | Date | Change |
| --- | --- | --- |
| v1.0 | 4 Aug 2025 | OnePlus Pad 2 Pro only: accepted prjname `24926` or `24976`, copied `24976/cps8601_firmware.bin` into `24926/`, and forced `ro.boot.prjname=24976` through `system.prop` |
| v1.1 | 12 Aug 2025 | Added the `post-fs-data.sh` overlay mount |
| v1.2 | 19 Aug 2025 | Any OnePlus device; picks among several source directories; stopped forcing `ro.boot.prjname` |

**Caveats found in the code** (*verified by reading the scripts; the first relies on how Magisk's installer works*)

- **Re-flashing probably doesn't move to the next directory.** The marker files are written into the module folder, which Magisk empties at the start of every install. So the install most likely always takes the lowest-numbered directory, despite the "re-flash to select the next one" message.
- **False success when nothing else is found.** With no other five-digit directory, the count still comes out as one (an empty line), the copy finds no files, and the script prints success with an empty folder.
- **It can overwrite correct firmware.** On a tablet whose own directory already exists, it copies another variant's firmware into that directory, and the overlay hides the original.
- **It doesn't check that the firmware fits.** It doesn't compare versions or hashes, so a successful install doesn't prove the copied file is right. This only works because, per both authors, the firmware is the same across regions.

## Choosing a fix

- **Look before installing anything.** If the tablet's `ro.boot.prjname` already has a directory, neither module is needed, and jjhitel's could make things worse.
- **jjhitel's module copies only what the ROM has.** If the only other directory holds firmware for different hardware, it will copy that.
- **cxzlw's module carries fixed copies.** It covers `24926` but not `24976`, and it depends on the root manager mounting `odm/`. One XDA reply says the KernelSU version did not work.
- **Swapping `odm.img` changes far more than pen firmware.** The ODM partition holds many vendor-specific files, which is a plausible reason for the reported boot crashes. It also has to be redone after every system update. User reports are mixed: one success (ColorOS .501 ODM on OxygenOS .500 EU), one boot crash, and one pen that stopped charging after a later global update. *Unverified*
- Keep a full backup and a way back to stock before flashing partitions.

## Checking a tablet (read-only)

From PowerShell or any shell on the computer, with the tablet connected over USB debugging:

```powershell
adb shell getprop ro.boot.prjname
adb shell getprop ro.product.system.manufacturer
adb shell ls -lR /odm/firmware/wireless_pen/
adb shell ls /data/adb/modules
adb shell "su -c 'sha256sum /odm/firmware/wireless_pen/*/*.bin'"
adb shell "su -c 'mount | grep anyfs'"
adb shell "su -c 'dmesg | grep cps8601'"
```

| Command | What it tells you |
| --- | --- |
| `getprop ro.boot.prjname` | The project number the firmware directory must match |
| `ls -lR /odm/firmware/wireless_pen/` | Which directories exist now. With an overlay active, this shows the combined view |
| `ls /data/adb/modules` | Whether `fix-odm-oplus-pad2pro` or `oplus-global-pen-enabler` is installed |
| `sha256sum …` | Which firmware build each directory holds; compare with the table above |
| `mount \| grep anyfs` | Whether jjhitel's overlay is active |
| `dmesg \| grep cps8601`, run after attaching the pen | `Rx attach!`, `receive ask pkt!`, `get ble addr` and `report 1` mean the handshake worked. Stopping at `error_attach_check_work_func` suggests it failed |

The repository README says this app was tested on a Pad 3 converted from OPD2413 to OPD2415. If the pen already pairs there, one of these fixes or an equivalent is probably installed, and the commands will show which.

## Relevance to Stylo 2 haptics

- **Separate paths.** The CPS8601 handles charging, attach detection and handing over the pen's Bluetooth address. Haptic commands go from apps to `com.oplus.ipemanager`, then over Bluetooth LE to the pen. The chip firmware has no haptics content. *Verified (strings) and inferred (path)*
- **It is still a prerequisite.** On a converted tablet, haptics only work after this handshake succeeds.
- **It marks a capture point.** The address hand-over is when pairing starts, so an HCI snoop log taken while attaching the pen should show the Bluetooth LE connection to the accessory service being set up.
- Neither fix module touches haptics or exposes the chip to apps.

## Open questions

- How exactly does `oplus_wireless_pen` build the firmware path? The GPL source in OnePlus's kernel release should show it.
- Which project number is Chinese and which global, for each pair? What does the converted Pad 3 report?
- What changed between the `239xx` firmware and `24926`, and how does `24976` compare? The `ver%bu.%bu` line in the kernel log would show versions.
- Does the KernelSU failure come from KernelSU not mounting a module's top-level `odm/` folder?

## Sources

- jjhitel, [fix-odm-oplus-pad2pro](https://github.com/jjhitel/fix-odm-oplus-pad2pro): scripts, changelog and history at commit `0131a9d`, plus the uploaded v1.2 zip (SHA-256 `9371189d2811bcd7696685dd5d412b6f6efa0c7adcbbcb1d4d2ef48116484331`). Read 2 Nov 2026.
- cxzlw, [oplus-global-pen-enabler](https://github.com/cxzlw/oplus-global-pen-enabler): README (module info, kernel logs, method), `module.prop` and firmware files at commit `1869c22`. Read 2 Nov 2026.
- crDroid, [android_device_oneplus_caihong `proprietary-files.txt`](https://github.com/crdroidandroid/android_device_oneplus_caihong/blob/16.0/proprietary-files.txt), "Wireless pen firmware" section.
- XDA, not readable automatically (search excerpts only):
  - [Stylus Fix for OnePlus on Different Regional ROM](https://xdaforums.com/t/magisk-module-stylus-fix-for-oneplus-on-different-regional-rom.4753007/)
  - [Found the way to fix stylus for OnePlus Pad 3 COS to OOS](https://xdaforums.com/t/found-the-way-to-fix-stylus-for-oneplus-pad-3-cos-to-oos.4749818/)
  - [Stylus not recognized on OPPO Pad 4 Pro after converting to OPD2415](https://xdaforums.com/t/title-q-stylus-not-recognized-on-oppo-pad-4-pro-opd2413-after-converting-to-oneplus-pad-3-global-rom-opd2415.4780653/)
