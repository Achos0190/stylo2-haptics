#!/usr/bin/env python3
"""Compare the pen service's installed binder interface with this repo's reconstruction.

Part of the owner-approved interface check (PICKUP.md Step 2b). The owner
allowed decompiling the copy of com.oplus.ipemanager pulled from their own
tablet, limited to the pen-SDK interface classes, for interoperability only.
This tool reads the decompiled Java (for example JADX output under
research/local/) and reports, for each interface:

    - whether its descriptor string is present
    - each method's transaction code and signature, installed vs reconstructed
    - methods that are missing, extra, renumbered or have a different signature

It records only interface facts: descriptor, method names, transaction codes
and parameter/return types. It never copies code. The JSON it writes is safe to
quote in research records; the decompiled sources stay in research/local/.

Usage:
    python3 research/tools/aidl_check.py <decompiled-dir-or-file> [...]
    python3 research/tools/aidl_check.py <dir> --json research/local/pad3/aidl-check.json
    python3 research/tools/aidl_check.py --reference-only      # print the reconstruction

The reference is the reconstruction in app/src/main/java/com/oplus/ipemanager/sdk/
(the PoC carries an identical copy). Standard library only.
"""
import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
REF_DIR = REPO / "app" / "src" / "main" / "java" / "com" / "oplus" / "ipemanager" / "sdk"
DEFAULT_JSON = REPO / "research" / "local" / "pad3" / "aidl-check.json"
INTERFACES = {
    "com.oplus.ipemanager.sdk.ISdkAidlInterface": "ISdkAidlInterface.java",
    "com.oplus.ipemanager.sdk.ISDKAidlCallback": "ISDKAidlCallback.java",
}

# An abstract interface method: "<ret> <name>(<params>) throws [android.os.]RemoteException;"
DECL_RE = re.compile(
    r"(?m)^[ \t]*(?:public\s+)?(?:abstract\s+)?([\w.$<>\[\], ?]+?)\s+(\w+)\s*\(([^)]*)\)\s*"
    r"throws\s+(?:android\.os\.)?RemoteException\s*;")
# A method with a body (Proxy implementation or Stub dispatch helper).
IMPL_RE = re.compile(
    r"(?m)^[ \t]*(?:@Override\s+)?(?:public\s+)?(?:final\s+)?([\w.$<>\[\], ?]+?)\s+(\w+)\s*\(([^)]*)\)\s*"
    r"(?:throws\s+[\w.]+(?:\s*,\s*[\w.]+)*\s*)?\{")
CONST_RE = re.compile(r"TRANSACTION_(\w+)\s*=\s*(\d+)\s*;")
TRANSACT_RE = re.compile(r"\.transact\(\s*(?:\w+\.)*(?:TRANSACTION_(\w+)|(\d+))\s*,")
CASE_RE = re.compile(r"case\s+(?:\w+\.)*(?:TRANSACTION_(\w+)|(\d+))\s*:")


def norm_type(t):
    t = re.sub(r"\b(?:java\.util|java\.lang|android\.os|android\.content)\.", "", t.strip())
    t = re.sub(r"\s+", "", t)
    return t.replace("final", "")


def param_types(params):
    out = []
    for p in [x.strip() for x in params.split(",") if x.strip()]:
        p = re.sub(r"@\w+(\([^)]*\))?\s*", "", p)
        p = re.sub(r"^final\s+", "", p)
        parts = p.rsplit(None, 1)
        out.append(norm_type(parts[0] if len(parts) == 2 else p))
    return out


def signature(ret, name, params):
    return f"{norm_type(ret)} {name}({', '.join(param_types(params))})"


def body_after(text, start):
    """Return the brace-balanced body starting at the '{' at or after start."""
    i = text.find("{", start)
    if i < 0:
        return ""
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[i + 1:j]
    return text[i + 1:]


def parse_interface(text):
    """Extract methods {name: {"signature", "code"}} from Java source of one interface."""
    methods = {}
    for m in DECL_RE.finditer(text):
        ret, name, params = m.groups()
        if name in ("asBinder", "onTransact"):
            continue
        methods.setdefault(name, {"signature": signature(ret, name, params), "code": None})
    consts = {name: int(code) for name, code in CONST_RE.findall(text)}
    for name, info in methods.items():
        if name in consts:
            info["code"] = consts[name]
    # Proxy bodies: "<method>(...) { ... transact(N or TRANSACTION_x, ...) }"
    for m in IMPL_RE.finditer(text):
        ret, name, params = m.groups()
        if name not in methods or methods[name]["code"] is not None:
            continue
        t = TRANSACT_RE.search(body_after(text, m.end() - 1))
        if t:
            const_name, num = t.groups()
            code = consts.get(const_name) if const_name else int(num)
            if code is not None:
                methods[name]["code"] = code
    # Stub dispatch: "case N: ... name(...)"
    unresolved = [n for n, i in methods.items() if i["code"] is None]
    if unresolved:
        ot = re.search(r"onTransact\s*\(", text)
        if ot:
            body = body_after(text, ot.end())
            cases = list(CASE_RE.finditer(body))
            for k, c in enumerate(cases):
                end = cases[k + 1].start() if k + 1 < len(cases) else len(body)
                seg = body[c.end():end]
                const_name, num = c.groups()
                code = consts.get(const_name) if const_name else int(num)
                for n in unresolved:
                    if code is not None and re.search(rf"\b{re.escape(n)}\s*\(", seg) and methods[n]["code"] is None:
                        methods[n]["code"] = code
                        break
    return methods


def load_reference():
    ref = {}
    for desc, fname in INTERFACES.items():
        p = REF_DIR / fname
        if not p.exists():
            sys.exit(f"Reference not found: {p}")
        text = p.read_text(encoding="utf-8")
        ref[desc] = {"file": str(p.relative_to(REPO)), "descriptor_present": desc in text,
                     "methods": parse_interface(text)}
    return ref


def java_files(targets):
    for t in targets:
        p = Path(t)
        if p.is_dir():
            yield from sorted(p.rglob("*.java"))
        elif p.is_file():
            yield p
        else:
            sys.exit(f"Not found: {t}")


def scan(targets):
    """Find the installed interfaces by descriptor string in decompiled sources."""
    found = {d: {"files": [], "methods": {}} for d in INTERFACES}
    for f in java_files(targets):
        try:
            text = f.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for desc in INTERFACES:
            if f'"{desc}"' in text:
                found[desc]["files"].append(f.name)
                for name, info in parse_interface(text).items():
                    cur = found[desc]["methods"].get(name)
                    if cur is None or (cur["code"] is None and info["code"] is not None):
                        found[desc]["methods"][name] = info
    return found


def compare(ref_methods, got_methods):
    rows = []
    for name in sorted(set(ref_methods) | set(got_methods),
                       key=lambda n: ((got_methods.get(n) or ref_methods.get(n))["code"] or 999, n)):
        r, g = ref_methods.get(name), got_methods.get(name)
        if r and not g:
            status = "missing in installed"
        elif g and not r:
            status = "extra in installed"
        elif r["code"] != g["code"] and r["signature"] != g["signature"]:
            status = "code and signature differ"
        elif r["code"] != g["code"]:
            status = "code differs"
        elif r["signature"] != g["signature"]:
            status = "signature differs"
        else:
            status = "match"
        rows.append({"method": name, "status": status,
                     "reconstructed": r, "installed": g})
    # By code, so renamed (obfuscated) methods still line up.
    by_code = {}
    for side, ms in (("reconstructed", ref_methods), ("installed", got_methods)):
        for name, info in ms.items():
            if info["code"] is not None:
                by_code.setdefault(info["code"], {})[side] = info["signature"]
    return rows, {str(k): v for k, v in sorted(by_code.items())}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("targets", nargs="*", help="decompiled source directories or .java files")
    ap.add_argument("--json", default=str(DEFAULT_JSON), help=f"output JSON (default {DEFAULT_JSON})")
    ap.add_argument("--reference-only", action="store_true")
    ap.add_argument("--apk-version", default="", help="installed versionName, recorded in the JSON")
    a = ap.parse_args(argv)
    ref = load_reference()
    if a.reference_only or not a.targets:
        for desc, r in ref.items():
            print(desc)
            for name, i in sorted(r["methods"].items(), key=lambda x: (x[1]["code"] or 999, x[0])):
                print(f"  {i['code'] if i['code'] is not None else '-':>3}  {i['signature']}")
        if not a.targets and not a.reference_only:
            print("\nPass the decompiled source directory to compare.")
        return 0

    found = scan(a.targets)
    result = {"apk_version": a.apk_version, "interfaces": {}}
    differences = 0
    for desc in INTERFACES:
        rows, by_code = compare(ref[desc]["methods"], found[desc]["methods"])
        present = bool(found[desc]["files"])
        bad = [r for r in rows if r["status"] != "match"]
        differences += len(bad) + (0 if present else 1)
        result["interfaces"][desc] = {
            "descriptor_found": present,
            "found_in_files": sorted(set(found[desc]["files"])),
            "methods": rows,
            "by_code": by_code,
            "summary": ("descriptor not found" if not present else
                        f"{sum(r['status'] == 'match' for r in rows)} match, {len(bad)} differ"),
        }
        print(f"\n{desc}: {result['interfaces'][desc]['summary']}")
        if present:
            print(f"  found in: {', '.join(sorted(set(found[desc]['files'])))}")
            print(f"  {'code':>4}  {'status':26}  method")
            for r in rows:
                i = r["installed"] or r["reconstructed"]
                code = i["code"] if i["code"] is not None else "-"
                print(f"  {code:>4}  {r['status']:26}  {i['signature']}")
                if r["status"] in ("code differs", "signature differs", "code and signature differ"):
                    print(f"  {'':>4}  {'':26}  reconstructed: {r['reconstructed']['code']} "
                          f"{r['reconstructed']['signature']}")
            if sum(r["status"] == "missing in installed" for r in rows) > len(ref[desc]["methods"]) // 2:
                print("  Most names don't match (renamed/obfuscated?). Lined up by transaction code:")
                for code, sides in by_code.items():
                    print(f"  {code:>4}  reconstructed: {sides.get('reconstructed', '-'):40}  "
                          f"installed: {sides.get('installed', '-')}")
    out = Path(a.json)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(f"\n-> {out}")
    print("Record these interface facts in gap-aidl-matches-firmware and if-oplus-sdk-interface. "
          "Don't change the app's codes or descriptor without the owner reviewing this result.")
    return 1 if differences else 0


if __name__ == "__main__":
    sys.exit(main())
