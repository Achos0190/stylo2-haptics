# Provenance: fix-odm-oplus-pad2pro v1.2

An unmodified reference copy of jjhitel's "Stylus Fixes for OnePlus" Magisk module scripts. It is kept so the analysis in [`../../docs/pen-attach-handshake.md`](../../docs/pen-attach-handshake.md) can be checked against the exact code.

**This folder is not an installable module.** It is reference material only.

| | |
| --- | --- |
| Author | jjhitel (Jeong Seongpil) |
| Licence | GPL-3.0, see [`LICENSE`](LICENSE) |
| Upstream | https://github.com/jjhitel/fix-odm-oplus-pad2pro |
| Upstream commit | `0131a9d7cc414715e9469f092828510138bafacf` (19 Aug 2025) |
| Release zip | `fix-odm-oplus-pad2pro_v1.2.zip`, 3,306 bytes, SHA-256 `9371189d2811bcd7696685dd5d412b6f6efa0c7adcbbcb1d4d2ef48116484331` |
| Copied | 2 Nov 2026 |

The zip's scripts are byte-identical to the upstream commit. The only difference is that the zip's `module.prop` uses Windows (CRLF) line endings; this copy keeps the repository's.

| File | SHA-256 |
| --- | --- |
| `customize.sh` | `258ed97560e5af099163f52351586f0d0cb7a92a3f183fb62bf1fc89ca9b9cde` |
| `post-fs-data.sh` | `5b4048abe14057a6e84ccfdc9a704323d8ca498f1305ef5803ab407f7c80f52e` |
| `module.prop` (LF) | `a6eeb594f19f55cc36994f632b5d2c1832afd976c82e00276ad7244914fcae25` |
| `META-INF/com/google/android/update-binary` | `b4c882fffba5b85804f6212b1bdf9ca68437e5150dd3a50f869ef115f4af5380` |
| `META-INF/com/google/android/updater-script` | `d2b8203193a083e43ad076fde01b9c7821e0aa521e1f8077b8628fb1cdb70c60` |

Not copied: `odm/firmware/wireless_pen/24926/.placeholder`, an empty file. The module ships no firmware.
