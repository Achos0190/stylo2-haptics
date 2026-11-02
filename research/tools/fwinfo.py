#!/usr/bin/env python3
"""Inspect CPS8601 pen-charger firmware files (cps8601_firmware.bin).

For each file: size, SHA-256, match against known builds, how much of the
image is used, whether it starts with an 8051 vector table, and its readable
strings. Read-only; uses only the Python standard library.

Usage:
    python3 research/tools/fwinfo.py <file-or-directory> [...]
    python3 research/tools/fwinfo.py --strings <file>        # also list strings
    python3 research/tools/fwinfo.py --diff <file-a> <file-b>
    python3 research/tools/fwinfo.py --json <file-or-directory> [...]

Directories are searched recursively for *.bin files. Keep firmware pulled
from a tablet in research/local/ (ignored by git); commit only hashes and
findings.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

# Known builds, from research/reference/oplus-global-pen-enabler/MANIFEST.md.
KNOWN = {
    "cfac67370d543b8b31a914a240dca5b043743388da539465260e6df6400f9e81":
        "239xx build (cxzlw module: 23926, 23927, 23976, 23978)",
    "33a4ab21db5e92af7f144fecf3d86f64137079d22e4890902d6da24dd6aa1ee4":
        "24926 build (cxzlw module, added Apr 2026)",
}

# 8051 vector slots: reset, external interrupt 0, timer 0. LJMP opcode is 0x02.
VECTORS = {0x0000: "reset", 0x0003: "INT0", 0x000B: "timer0"}
STR_RE = re.compile(rb"[\x20-\x7e]{6,}")


def used_length(b):
    end = len(b)
    while end > 0 and b[end - 1] in (0x00, 0xFF):
        end -= 1
    return end


def vectors(b):
    out = {}
    for off, name in VECTORS.items():
        if off + 3 <= len(b) and b[off] == 0x02:
            out[name] = f"LJMP 0x{b[off + 1]:02X}{b[off + 2]:02X}"
        else:
            out[name] = None
    return out


def strings(b):
    found = []
    for m in STR_RE.finditer(b):
        s = m.group().decode("ascii")
        if re.search(r"[a-z]{3}", s):          # skip opcode noise without words
            found.append(s.strip())
    return found


def info(path):
    b = path.read_bytes()
    digest = hashlib.sha256(b).hexdigest()
    vec = vectors(b)
    return {
        "path": str(path),
        "bytes": len(b),
        "sha256": digest,
        "known": KNOWN.get(digest, "unknown build"),
        "used_bytes": used_length(b),
        "looks_8051": all(vec.values()),
        "vectors": vec,
        "strings": strings(b),
        "version_format_present": b"ver%bu.%bu" in b,
    }


def files(args):
    for a in args:
        p = Path(a)
        if p.is_dir():
            yield from sorted(x for x in p.rglob("*.bin") if x.is_file())
        elif p.is_file():
            yield p
        else:
            sys.exit(f"Not found: {a}")


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return
    if argv[0] == "--diff":
        if len(argv) != 3:
            sys.exit("--diff needs two files")
        a, b = Path(argv[1]).read_bytes(), Path(argv[2]).read_bytes()
        n = min(len(a), len(b))
        diff = sum(1 for i in range(n) if a[i] != b[i]) + abs(len(a) - len(b))
        first = next((i for i in range(n) if a[i] != b[i]), None)
        print(f"{diff} differing bytes; sizes {len(a)} and {len(b)}; first difference at "
              + ("none" if first is None else f"0x{first:04X}"))
        return
    as_json = argv[0] == "--json"
    show_strings = argv[0] == "--strings"
    targets = argv[1:] if (as_json or show_strings) else argv
    results = [info(p) for p in files(targets)]
    if as_json:
        print(json.dumps(results, indent=2))
        return
    for r in results:
        print(r["path"])
        print(f"  {r['bytes']} bytes, {r['used_bytes']} used, sha256 {r['sha256']}")
        print(f"  build: {r['known']}")
        vec = ", ".join(f"{k} {v or '-'}" for k, v in r["vectors"].items())
        print(f"  8051 vectors: {'yes' if r['looks_8051'] else 'no'} ({vec})")
        print(f"  {len(r['strings'])} strings; version format string "
              f"{'present' if r['version_format_present'] else 'absent'}")
        if show_strings:
            for s in r["strings"]:
                print(f"    {s}")


if __name__ == "__main__":
    main(sys.argv[1:])
