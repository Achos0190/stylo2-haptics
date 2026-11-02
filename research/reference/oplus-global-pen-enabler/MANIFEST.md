# Manifest: oplus-global-pen-enabler firmware files

A record of the CPS8601 pen-charger firmware files in cxzlw's KernelSU module, so that a copy found on a tablet or in a ROM can be identified.

**The binaries are not stored here.** They are OnePlus firmware, and the upstream repository states no licence that would allow redistribution. Get them from the upstream repository, or read them from your own tablet's `/odm/firmware/wireless_pen/`.

| | |
| --- | --- |
| Author | cxzlw, with a later contribution from FRTI64 |
| Licence | None stated |
| Upstream | https://github.com/cxzlw/oplus-global-pen-enabler |
| Upstream commit | `1869c22bfb2098c70dc15ea059f02608b167f3d0` (12 Apr 2026) |
| Recorded | 2 Nov 2026 |

## Files

| Path | Bytes | SHA-256 | Added in |
| --- | ---: | --- | --- |
| `odm/firmware/wireless_pen/23926/cps8601_firmware.bin` | 15,360 | `cfac67370d543b8b31a914a240dca5b043743388da539465260e6df6400f9e81` | `3197da8` (6 Mar 2025) |
| `odm/firmware/wireless_pen/23927/cps8601_firmware.bin` | 15,360 | `cfac67370d543b8b31a914a240dca5b043743388da539465260e6df6400f9e81` | `3197da8` |
| `odm/firmware/wireless_pen/23976/cps8601_firmware.bin` | 15,360 | `cfac67370d543b8b31a914a240dca5b043743388da539465260e6df6400f9e81` | `3197da8` |
| `odm/firmware/wireless_pen/23978/cps8601_firmware.bin` | 15,360 | `cfac67370d543b8b31a914a240dca5b043743388da539465260e6df6400f9e81` | `3197da8` |
| `odm/firmware/wireless_pen/24926/cps8601_firmware.bin` | 15,360 | `33a4ab21db5e92af7f144fecf3d86f64137079d22e4890902d6da24dd6aa1ee4` | `1869c22` (12 Apr 2026) |

The two builds differ in 9,675 of their 15,360 bytes. Both are padded with zeros after their last code or data byte (offset 14,972 in the `239xx` build, 15,065 in `24926`).

## module.prop

```
id=oplus-global-pen-enabler
name=OnePlus 手写笔修复
version=0
versionCode=0
author=cxzlw
description=为 OnePlus Pad Pro 用户在国际版 ROM 恢复国行手写笔的可用性
```

In English, the name is "OnePlus stylus fix" and the description is "Restores the Chinese-market stylus on international ROMs for OnePlus Pad Pro users".

The module has no install or boot scripts. It relies on the root manager mounting its top-level `odm/` folder over `/odm`.

See [`../../docs/pen-attach-handshake.md`](../../docs/pen-attach-handshake.md) for the firmware analysis.
