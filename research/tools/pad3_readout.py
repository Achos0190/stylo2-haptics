#!/usr/bin/env python3
"""Read-only ADB read-out of a OnePlus tablet for the Stylo 2 research.

Collects, from the tablet connected over ADB, everything the local-session
playbook (PICKUP.md) asks for: identity, the pen-charger firmware files
(listed, hashed and pulled), installed root modules and overlays, the
wireless-pen kernel driver, the pen service package and its permission, the
pen service APK itself (pulled and hashed, for the owner-approved interface
check), the Bluetooth bond, the attach handshake log, and the Route-A PoC
app's log.

Everything it runs on the tablet is a read. It refuses to run any device
command that is not on its read-only allowlist. The single exception is a
temporary copy under /sdcard/Download/stylo2-readout-* used to pull a file
that the shell user cannot read directly; it is deleted straight away.

Root is optional. With --root auto (the default) it uses `su` for reads when
the tablet is rooted and skips root-only reads otherwise, saying so.

Output (default research/local/pad3/, which git ignores):
    raw/<step>/...      unredacted command output and pulled files (never commit)
    <step>.json         structured, redacted findings for each step
    summary.md          redacted summary to quote in research records and docs

Usage:
    python3 research/tools/pad3_readout.py check
    python3 research/tools/pad3_readout.py all            # every non-interactive step
    python3 research/tools/pad3_readout.py firmware       # one step
    python3 research/tools/pad3_readout.py apk            # pull the pen service APK
    python3 research/tools/pad3_readout.py handshake      # guided pen attach/detach
    python3 research/tools/pad3_readout.py poc-log        # after running the PoC app
    python3 research/tools/pad3_readout.py summary        # rebuild summary.md

Options:
    --adb PATH        adb executable (default: adb on PATH)
    -s SERIAL         device serial, when more than one is connected
    --out DIR         output directory (default: research/local/pad3)
    --root MODE       auto | yes | no   (default: auto)
    --no-prompt       don't wait for Enter or ask questions (handshake, poc-log)
    --packages LIST   apk step: comma-separated packages to pull
                      (default: com.oplus.ipemanager)

On Windows run it as `py -3 research/tools/pad3_readout.py ...`.
Standard library only.
"""
import argparse
import datetime as _dt
import hashlib
import importlib.util
import json
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
REPO = TOOLS.parent.parent
DEFAULT_OUT = REPO / "research" / "local" / "pad3"

FW_DIR = "/odm/firmware/wireless_pen"
FW_OVERLAY = "/dev/anyfs/upper/odm/firmware/wireless_pen"   # jjhitel's overlay
TMP_PREFIX = "/sdcard/Download/stylo2-readout-"
PEN_SERVICE_PKG = "com.oplus.ipemanager"
PEN_PERMISSION = "com.oplus.permission.safe.IOT"
PEN_SDK_ACTION = "com.oplus.ipemanager.ACTION.PENCIL_SDK"
POC_PKG = "io.github.yeetingwaterbottle.stylo2.poc"
POC_LOG_TAG = "Stylo2Poc"
UPSTREAM_APP_PKG = "io.github.yeetingwaterbottle.stylo2.haptics"
INSPECTOR_PKG = "io.github.yeetingwaterbottle.stylo2.bleinspector"
SHIZUKU_PKG = "moe.shizuku.privileged.api"

IDENTITY_PROPS = [
    "ro.boot.prjname", "ro.product.model", "ro.product.device", "ro.product.name",
    "ro.product.system.manufacturer", "ro.product.brand", "ro.build.display.id",
    "ro.build.version.release", "ro.build.version.sdk", "ro.build.version.incremental",
    "ro.build.version.security_patch", "ro.build.type", "ro.build.fingerprint",
    "ro.boot.verifiedbootstate", "ro.boot.flash.locked", "ro.boot.vbmeta.device_state",
    "persist.bluetooth.btsnooplogmode",
]
# Properties worth a look when telling Chinese from global hardware.
REGION_PROP_RE = re.compile(
    r"prjname|region|regionmark|hw_|nv_id|oplus\.version|ota|rf_version|operator|carrier|euex",
    re.I)
# Handshake markers from the cxzlw kernel log (research/docs/pen-attach-handshake.md).
HANDSHAKE_MARKERS = [
    "Rx attach", "tx wakeup", "receive ask pkt", "get ble addr", "cps8601_valid_check",
    "report 1", "error_attach_check_work_func",
]
DRIVER_STRING_RE = re.compile(r"wireless_pen|cps8601|firmware|prj|project|%s|%d", re.I)

# ---------------------------------------------------------------- redaction

MAC_RE = re.compile(r"\b(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}\b")
PARTIAL_MAC_RE = re.compile(r"\b(?:XX:){2,4}[0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2})*\b", re.I)
IMEI_RE = re.compile(r"(?<!\d)\d{15}(?!\d)")
EMAIL_RE = re.compile(r"[\w.+-]+@[\w-]+\.[\w.-]+")
SECRET_PROP_RE = re.compile(r"serial|imei|meid|iccid|msisdn|\bsn\b|account|wifimac|btmac|bt_addr|mac_addr", re.I)
GET_BLE_ADDR_RE = re.compile(r"(get ble addr)\s*\([^)]*\)", re.I)


def redact(text):
    """Remove Bluetooth/Wi-Fi addresses, IMEIs, emails and the handshake address."""
    if not text:
        return text
    text = GET_BLE_ADDR_RE.sub(r"\1(<redacted>)", text)
    text = MAC_RE.sub("<redacted-mac>", text)
    text = PARTIAL_MAC_RE.sub("<redacted-mac>", text)
    text = IMEI_RE.sub("<redacted-number>", text)
    return EMAIL_RE.sub("<redacted-email>", text)


def redact_prop(key, value):
    if SECRET_PROP_RE.search(key):
        return "<redacted>"
    return redact(value)


# ---------------------------------------------------------------- read-only guard

class NotReadOnly(Exception):
    pass


def _tmp_path(p):
    return p.startswith(TMP_PREFIX) and ".." not in p and len(p) > len(TMP_PREFIX)


def check_readonly(argv):
    """Raise NotReadOnly unless argv is a read on the allowlist."""
    if not argv:
        raise NotReadOnly("empty command")
    cmd, args = argv[0], argv[1:]
    flags = [a for a in args if a.startswith("-")]

    def bad(why):
        raise NotReadOnly(f"{' '.join(argv)}: {why}")

    if cmd in ("getprop", "ls", "cat", "sha256sum", "modinfo", "lsmod", "id", "uname", "mount"):
        if cmd == "mount" and args:
            bad("mount only without arguments (list mounts)")
        if cmd == "getprop" and len(args) > 1:
            bad("getprop takes at most one property")
        return
    if cmd == "date":
        if args != ["+%s"]:
            bad("date only as 'date +%s'")
        return
    if cmd == "find":
        if any(a in ("-delete", "-exec", "-execdir", "-ok", "-okdir", "-fprint", "-fprintf", "-fls")
               for a in args):
            bad("find may not delete, exec or write")
        return
    if cmd == "grep":
        return
    if cmd == "pm":
        if not args or args[0] not in ("path", "list"):
            bad("pm only 'path' or 'list'")
        return
    if cmd == "dumpsys":
        allowed = (["package"], ["activity", "services"], ["bluetooth_manager"])
        if not any(args[:len(a)] == a for a in allowed):
            bad("dumpsys only package / activity services / bluetooth_manager")
        if any(a in ("set", "reset", "unplug", "clear", "--reset", "enable", "disable") for a in args):
            bad("dumpsys argument changes state")
        return
    if cmd == "logcat":
        if "-d" not in args:
            bad("logcat must use -d (dump and exit)")
        if any(f in ("-c", "--clear", "-G", "--buffer-size", "-P", "--prune", "-f", "--file", "-r", "-n")
               or f.startswith(("-G", "-P", "-f")) and f not in ("-d",)
               for f in flags):
            bad("logcat flag changes state or writes files")
        return
    if cmd == "dmesg":
        if any(f in ("-c", "-C", "-D", "-E", "-n", "--clear", "--read-clear", "--console-off",
                     "--console-on", "--console-level") for f in flags):
            bad("dmesg flag changes state")
        return
    if cmd == "cp":
        if not args or not _tmp_path(args[-1]):
            bad(f"cp only into {TMP_PREFIX}*")
        return
    if cmd == "rm":
        targets = [a for a in args if not a.startswith("-")]
        if not targets or not all(_tmp_path(t) for t in targets):
            bad(f"rm only under {TMP_PREFIX}*")
        return
    bad("not on the read-only allowlist")


# ---------------------------------------------------------------- adb wrapper

class Adb:
    def __init__(self, exe="adb", serial=None, root_mode="auto"):
        self.exe = exe
        self.serial = serial
        self.root_mode = root_mode
        self._root = None

    def _base(self):
        return [self.exe] + (["-s", self.serial] if self.serial else [])

    def host(self, *args, timeout=60):
        """Run an adb host command (devices, pull, ...)."""
        try:
            p = subprocess.run(self._base() + list(args), capture_output=True, timeout=timeout)
        except FileNotFoundError:
            sys.exit(f"adb not found ({self.exe}). Install platform-tools or pass --adb.")
        except subprocess.TimeoutExpired:
            return 124, "", f"timed out after {timeout}s"
        return p.returncode, p.stdout.decode("utf-8", "replace"), p.stderr.decode("utf-8", "replace")

    def shell(self, argv, root=False, quiet=False, timeout=60):
        """Run a read-only command on the device. Returns (code, stdout, stderr)."""
        check_readonly(argv)
        cmd = " ".join(a if re.fullmatch(r"[\w@%+=:,./-]+", a) else shlex.quote(a) for a in argv)
        if quiet:
            cmd += " 2>/dev/null"
        if root:
            if not self.has_root():
                return 126, "", "root not available"
            cmd = "su -c " + shlex.quote(cmd)
        return self.host("shell", cmd, timeout=timeout)

    def read(self, argv, quiet=False, timeout=60):
        """Try as the shell user, then with root if available and the first try failed."""
        code, out, err = self.shell(argv, quiet=quiet, timeout=timeout)
        if (code != 0 or not out.strip()) and self.has_root():
            code2, out2, err2 = self.shell(argv, root=True, quiet=quiet, timeout=timeout)
            if code2 == 0 or out2.strip():
                return code2, out2, err2, True
        return code, out, err, False

    def has_root(self):
        if self._root is None:
            if self.root_mode == "no":
                self._root = False
            else:
                # `su -c id` is a read; approve the prompt on the tablet if one appears.
                code, out, _ = self.host("shell", "su -c id", timeout=30)
                self._root = code == 0 and "uid=0" in out
                if self.root_mode == "yes" and not self._root:
                    sys.exit("--root yes, but `su -c id` did not return uid=0. "
                             "Approve the su prompt on the tablet, or use --root auto.")
        return self._root

    def pull(self, remote, local, label):
        """Pull a file or directory. Falls back to a temporary root copy on /sdcard."""
        local = Path(local)
        local.parent.mkdir(parents=True, exist_ok=True)
        code, out, err = self.host("pull", remote, str(local), timeout=300)
        if code == 0:
            return {"pulled": True, "via": "adb pull", "local": str(local)}
        result = {"pulled": False, "via": None, "error": (err or out).strip()[-300:]}
        if not self.has_root():
            result["note"] = "direct pull refused and root not available"
            return result
        tmp = TMP_PREFIX + label
        self.shell(["rm", "-rf", tmp], root=True, quiet=True)
        c1, _, e1 = self.shell(["cp", "-r", remote, tmp], root=True)
        if c1 != 0:
            result["error"] += f"; root copy failed: {e1.strip()[-200:]}"
            return result
        try:
            c2, o2, e2 = self.host("pull", tmp, str(local), timeout=300)
        finally:
            self.shell(["rm", "-rf", tmp], root=True, quiet=True)
        if c2 == 0:
            return {"pulled": True, "via": "root copy to /sdcard/Download, pulled, copy deleted",
                    "local": str(local)}
        result["error"] += f"; pull of root copy failed: {(e2 or o2).strip()[-200:]}"
        return result


# ---------------------------------------------------------------- helpers

def today():
    return _dt.date.today().isoformat()


def now():
    return _dt.datetime.now().astimezone().isoformat(timespec="seconds")


def load_fwinfo():
    spec = importlib.util.spec_from_file_location("fwinfo", TOOLS / "fwinfo.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def ascii_strings(data, min_len=4):
    return [m.group().decode("ascii") for m in re.finditer(rb"[\x20-\x7e]{%d,}" % min_len, data)]


def parse_getprop(text):
    props = {}
    for line in text.splitlines():
        m = re.match(r"\[(.+?)\]: \[(.*)\]$", line.strip())
        if m:
            props[m.group(1)] = m.group(2)
    return props


def filter_lines(text, pattern, limit=400):
    rx = re.compile(pattern, re.I)
    return [ln for ln in text.splitlines() if rx.search(ln)][:limit]


class Run:
    """Shared state for one invocation: adb, output paths, and step results."""

    def __init__(self, adb, out, prompt=True):
        self.adb = adb
        self.out = Path(out)
        self.prompt = prompt
        self.out.mkdir(parents=True, exist_ok=True)

    def raw(self, step, name, text):
        p = self.out / "raw" / step / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")
        return p

    def save(self, step, data):
        data = {"step": step, "collected": now(), **data}
        p = self.out / f"{step}.json"
        p.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"  -> {p.relative_to(REPO) if p.is_relative_to(REPO) else p}")
        return data

    def load(self, step):
        p = self.out / f"{step}.json"
        return json.loads(p.read_text(encoding="utf-8")) if p.exists() else None

    def ask(self, message, default=""):
        if not self.prompt:
            return default
        try:
            return input(message).strip()
        except EOFError:
            return default


def need_device(adb):
    code, out, err = adb.host("get-state", timeout=15)
    if code != 0 or out.strip() != "device":
        sys.exit("No authorised device. Run `adb devices`: the tablet must show as `device` "
                 f"(got: {(out or err).strip() or 'nothing'}).")


# ---------------------------------------------------------------- steps

def step_check(run):
    print("check: host tools, device connection, root")
    host = {}
    for tool, args in (("adb", ["version"]), ("java", ["-version"]), ("cargo", ["--version"]),
                       ("rustup", ["--version"])):
        exe = run.adb.exe if tool == "adb" else shutil.which(tool)
        if not exe:
            host[tool] = None
            continue
        try:
            p = subprocess.run([exe] + args, capture_output=True, timeout=30)
            text = (p.stdout or p.stderr).decode("utf-8", "replace")
            line = [ln for ln in text.strip().splitlines() if not ln.startswith("Picked up ")]
            host[tool] = line[0] if line else "present"
        except (OSError, subprocess.TimeoutExpired):
            host[tool] = None
    host["python"] = sys.version.split()[0]
    sdk = [k for k in ("ANDROID_HOME", "ANDROID_SDK_ROOT") if __import__("os").environ.get(k)]
    host["android_sdk_env"] = sdk or None

    code, out, err = run.adb.host("devices", "-l", timeout=15)
    lines = [redact(ln) for ln in out.splitlines()[1:] if ln.strip()]
    device = {"adb_devices": lines}
    state_code, state, _ = run.adb.host("get-state", timeout=15)
    device["state"] = state.strip() if state_code == 0 else "not connected"
    if device["state"] == "device":
        props = {p: run.adb.shell(["getprop", p])[1].strip() for p in
                 ("ro.product.model", "ro.boot.prjname", "ro.build.display.id",
                  "ro.build.version.release", "ro.boot.verifiedbootstate", "ro.build.type")}
        device["props"] = props
        device["root"] = run.adb.has_root()
        installed = {}
        for label, pkg in (("pen_service", PEN_SERVICE_PKG), ("poc_app", POC_PKG),
                           ("ble_inspector", INSPECTOR_PKG), ("upstream_companion_app", UPSTREAM_APP_PKG),
                           ("shizuku", SHIZUKU_PKG)):
            c, o, _ = run.adb.shell(["pm", "path", pkg])
            installed[label] = c == 0 and "package:" in o
        c, o, _ = run.adb.shell(["pm", "list", "packages"])
        installed["blueshark_like"] = sorted(ln.split(":", 1)[-1] for ln in o.splitlines()
                                             if "blueshark" in ln.lower())
        device["installed"] = installed
    data = run.save("check", {"host": host, "device": device})
    for k, v in host.items():
        print(f"  host {k:16} {v or 'MISSING'}")
    print(f"  device state     {device['state']}")
    if device["state"] == "device":
        for k, v in device["props"].items():
            print(f"  {k:28} {v}")
        print(f"  root (su)        {'yes' if device['root'] else 'no'}")
        for k, v in device["installed"].items():
            print(f"  installed {k:24} {v}")
    return data


def step_identify(run):
    print("identify: model, build, project number")
    code, out, _ = run.adb.shell(["getprop"])
    run.raw("identify", "getprop.txt", out)
    props = parse_getprop(out)
    picked = {k: redact_prop(k, props.get(k, "")) for k in IDENTITY_PROPS}
    region = {k: redact_prop(k, v) for k, v in sorted(props.items())
              if REGION_PROP_RE.search(k) and k not in picked}
    code, uname, _ = run.adb.shell(["uname", "-a"])
    data = {"props": picked, "region_candidates": region, "kernel": redact(uname.strip()),
            "property_count": len(props)}
    print(f"  ro.boot.prjname = {picked['ro.boot.prjname'] or '(empty)'}")
    print(f"  {picked['ro.product.model']} / {picked['ro.product.device']} / {picked['ro.build.display.id']}")
    return run.save("identify", data)


def _ls_tree(run, path, step, name):
    code, out, err, via_root = run.adb.read(["ls", "-lR", path], quiet=True)
    run.raw(step, name, out + (("\n# stderr\n" + err) if err.strip() else ""))
    return out, via_root


def _parse_fw_listing(text):
    """Folder names and symlinks from `ls -lR` of the firmware directory."""
    folders, links = [], []
    for ln in text.splitlines():
        parts = ln.split()
        if not parts:
            continue
        if ln.startswith("d"):
            folders.append(parts[-1])
        elif ln.startswith("l") and "->" in ln:
            i = parts.index("->")
            links.append({"name": parts[i - 1], "target": " ".join(parts[i + 1:])})
    return sorted(set(folders)), links


def step_firmware(run):
    print(f"firmware: list, hash and pull {FW_DIR}")
    fw = load_fwinfo()
    listing, listing_root = _ls_tree(run, FW_DIR, "firmware", "ls-wireless_pen.txt")
    folders, links = _parse_fw_listing(listing)

    code, found, _, _ = run.adb.read(["find", FW_DIR, "-name", "*.bin"], quiet=True)
    remote_bins = sorted(ln.strip() for ln in found.splitlines() if ln.strip().endswith(".bin"))
    device_hashes = {}
    if remote_bins:
        code, out, _, _ = run.adb.read(["sha256sum"] + remote_bins)
        for ln in out.splitlines():
            parts = ln.split()
            if len(parts) == 2 and re.fullmatch(r"[0-9a-f]{64}", parts[0]):
                device_hashes[parts[1]] = parts[0]

    # Other copies anywhere on ODM/vendor (another layout would show up here).
    code, other, _, _ = run.adb.read(["find", "/odm", "/vendor", "-iname", "*cps8601*"], quiet=True)
    other_paths = sorted(p.strip() for p in other.splitlines()
                         if p.strip() and not p.strip().startswith(FW_DIR + "/"))

    prjname = run.adb.shell(["getprop", "ro.boot.prjname"])[1].strip()

    overlay = None
    if run.adb.has_root():
        c, o, _ = run.adb.shell(["ls", "-lR", FW_OVERLAY], root=True, quiet=True)
        run.raw("firmware", "ls-anyfs-overlay.txt", o)
        overlay = {"path": FW_OVERLAY, "present": c == 0 and bool(o.strip()),
                   "folders": _parse_fw_listing(o)[0]}

    local_dir = run.out / "raw" / "firmware" / "wireless_pen"
    if local_dir.exists():
        shutil.rmtree(local_dir)
    pull = run.adb.pull(FW_DIR, local_dir, "wireless_pen")
    files = []
    if pull.get("pulled"):
        for p in sorted(local_dir.rglob("*.bin")):
            info = fw.info(p)
            rel = p.relative_to(local_dir).as_posix()
            remote = f"{FW_DIR}/{rel}"
            files.append({
                "path": remote,
                "folder": rel.split("/")[0] if "/" in rel else "",
                "bytes": info["bytes"],
                "used_bytes": info["used_bytes"],
                "sha256": info["sha256"],
                "device_sha256_matches": device_hashes.get(remote) in (None, info["sha256"]),
                "known_build": info["known"],
                "looks_8051": info["looks_8051"],
                "version_format_present": info["version_format_present"],
                "string_count": len(info["strings"]),
            })
            run.raw("firmware", f"strings-{rel.replace('/', '_')}.txt", "\n".join(info["strings"]) + "\n")
    else:
        # Not pulled: keep the on-device hashes so the build can still be identified.
        for remote, digest in sorted(device_hashes.items()):
            rel = remote[len(FW_DIR) + 1:]
            files.append({"path": remote, "folder": rel.split("/")[0], "sha256": digest,
                          "known_build": fw.KNOWN.get(digest, "unknown build"),
                          "note": "hashed on the tablet; file not pulled"})

    builds = {}
    for f in files:
        builds.setdefault(f["sha256"], []).append(f["folder"])
    data = {
        "firmware_dir": FW_DIR,
        "listing_needed_root": listing_root,
        "folders": folders,
        "symlinks": links,
        "prjname": prjname,
        "prjname_folder_present": prjname in folders,
        "files": files,
        "distinct_builds": {k: sorted(v) for k, v in builds.items()},
        "other_cps8601_paths": other_paths,
        "overlay": overlay,
        "pull": {k: v for k, v in pull.items() if k != "local"},
    }
    print(f"  folders: {', '.join(folders) or '(none found)'}; prjname {prjname or '?'} "
          f"{'HAS' if data['prjname_folder_present'] else 'has NO'} folder")
    for digest, where in data["distinct_builds"].items():
        print(f"  {digest[:12]}...  {fw.KNOWN.get(digest, 'unknown build')}  <- {', '.join(where)}")
    if not pull.get("pulled"):
        print(f"  pull failed: {pull.get('error') or pull.get('note')}")
    return run.save("firmware", data)


def step_modules(run):
    print("modules: root modules and overlays")
    if not run.adb.has_root():
        print("  skipped: needs root to read /data/adb")
        return run.save("modules", {"skipped": "root not available; /data/adb is not readable without it"})
    c, adb_dir, _ = run.adb.shell(["ls", "/data/adb"], root=True, quiet=True)
    managers = [n for n in adb_dir.split() if n in ("magisk", "ksu", "ap", "modules", "modules_update")]
    c, names, _ = run.adb.shell(["ls", "/data/adb/modules"], root=True, quiet=True)
    modules = []
    for name in names.split():
        c, prop, _ = run.adb.shell(["cat", f"/data/adb/modules/{name}/module.prop"], root=True, quiet=True)
        run.raw("modules", f"{name}.module.prop", prop)
        kv = dict(ln.split("=", 1) for ln in prop.splitlines() if "=" in ln)
        c, files, _ = run.adb.shell(["ls", f"/data/adb/modules/{name}"], root=True, quiet=True)
        flags = [f for f in files.split() if f in ("disable", "remove", "skip_mount", "update")]
        c, odm, _ = run.adb.shell(["find", f"/data/adb/modules/{name}/odm", "-name", "*.bin"],
                                  root=True, quiet=True)
        modules.append({
            "dir": name,
            "id": kv.get("id", ""), "name": kv.get("name", ""), "version": kv.get("version", ""),
            "versionCode": kv.get("versionCode", ""), "author": kv.get("author", ""),
            "flags": flags,
            "ships_firmware": sorted(p.split(f"/data/adb/modules/{name}/", 1)[-1]
                                     for p in odm.splitlines() if p.strip()),
        })
    c, mounts, _ = run.adb.shell(["mount"], root=True)
    run.raw("modules", "mount.txt", mounts)
    relevant = [redact(ln) for ln in filter_lines(mounts, r"anyfs|wireless_pen|\s/odm\b|/odm/")]
    data = {"root_managers": managers, "modules": modules, "relevant_mounts": relevant}
    for m in modules:
        print(f"  {m['dir']}: {m['name']} {m['version']} {' '.join(m['flags'])}"
              + (f" (firmware: {len(m['ships_firmware'])} files)" if m["ships_firmware"] else ""))
    if not modules:
        print("  no modules installed")
    return run.save("modules", data)


def step_driver(run):
    print("driver: oplus_wireless_pen.ko")
    c, lsmod, _, _ = run.adb.read(["cat", "/proc/modules"])
    loaded =[ln.split()[0] for ln in lsmod.splitlines() if ln.split() and "wireless_pen" in ln.split()[0]]
    c, found, _, _ = run.adb.read(
        ["find", "/vendor", "/vendor_dlkm", "/system_dlkm", "/odm", "/system/lib/modules",
         "-name", "oplus_wireless_pen.ko"], quiet=True, timeout=120)
    paths = sorted(set(p.strip() for p in found.splitlines() if p.strip().endswith(".ko")))
    c, kernel, _ = run.adb.shell(["uname", "-r"])
    data = {"loaded": loaded, "paths": paths, "kernel_release": kernel.strip(), "modinfo": {},
            "firmware_path_strings": []}
    if paths:
        c, info, _, _ = run.adb.read(["modinfo", paths[0]])
        run.raw("driver", "modinfo.txt", info)
        fields = {}
        for ln in info.splitlines():
            if ":" in ln:
                k, v = ln.split(":", 1)
                fields.setdefault(k.strip(), []).append(v.strip())
        data["modinfo"] = {k: v if len(v) > 1 else v[0] for k, v in fields.items()
                           if k in ("filename", "description", "alias", "depends", "vermagic",
                                    "license", "author", "version", "srcversion", "name", "firmware")}
        local = run.out / "raw" / "driver" / "oplus_wireless_pen.ko"
        pull = run.adb.pull(paths[0], local, "oplus_wireless_pen.ko")
        data["pull"] = {k: v for k, v in pull.items() if k != "local"}
        if pull.get("pulled"):
            blob = local.read_bytes()
            data["ko_sha256"] = sha256(local)
            data["ko_bytes"] = len(blob)
            hits = [s.strip() for s in ascii_strings(blob) if DRIVER_STRING_RE.search(s)]
            run.raw("driver", "strings-filtered.txt", "\n".join(hits) + "\n")
            data["firmware_path_strings"] = sorted(set(
                redact(s) for s in hits if re.search(r"wireless_pen|cps8601_firmware|firmware/", s, re.I)))[:80]
            data["prj_strings"] = sorted(set(redact(s) for s in hits if re.search(r"prj|project", s, re.I)))[:80]
    print(f"  loaded: {', '.join(loaded) or 'not listed'}; file: {', '.join(paths) or 'not found'}")
    if data["modinfo"]:
        print(f"  vermagic: {data['modinfo'].get('vermagic', '?')}")
    for s in data["firmware_path_strings"][:10]:
        print(f"  string: {s}")
    return run.save("driver", data)


def _parse_permissions(text, name):
    """Find one permission block in `pm list permissions -f` output."""
    block, inside = {}, False
    for ln in text.splitlines():
        s = ln.strip()
        if s.startswith("+ permission:") or s.startswith("permission:"):
            if inside:
                break
            inside = s.split("permission:", 1)[1] == name
            continue
        if inside and ":" in s:
            k, v = s.split(":", 1)
            block[k] = v
    return block


def step_service(run):
    print(f"service: {PEN_SERVICE_PKG} and {PEN_PERMISSION}")
    c, path, _ = run.adb.shell(["pm", "path", PEN_SERVICE_PKG])
    c, pkg, _ = run.adb.shell(["dumpsys", "package", PEN_SERVICE_PKG])
    run.raw("service", "dumpsys-package-ipemanager.txt", pkg)

    def first(rx):
        m = re.search(rx, pkg)
        return m.group(1) if m else ""

    data = {
        "package": PEN_SERVICE_PKG,
        "installed": "package:" in path,
        "apk_paths": [ln.split(":", 1)[1] for ln in path.splitlines() if ln.startswith("package:")],
        "versionName": first(r"versionName=(\S+)"),
        "versionCode": first(r"versionCode=(\d+)"),
        "lastUpdateTime": first(r"lastUpdateTime=([^\n]+)"),
        "mentions": {
            "CoreService": ".btadsorb.CoreService" in pkg,
            "PENCIL_SDK_action": PEN_SDK_ACTION in pkg,
            "safe.IOT": PEN_PERMISSION in pkg,
        },
        "relevant_lines": [redact(ln.strip()) for ln in
                           filter_lines(pkg, r"PENCIL_SDK|CoreService|safe\.IOT|versionName|versionCode", 60)],
    }

    c, perms, _ = run.adb.shell(["pm", "list", "permissions", "-f", "-g"], timeout=120)
    run.raw("service", "pm-list-permissions.txt", perms)
    block = _parse_permissions(perms, PEN_PERMISSION)
    data["permission"] = {
        "name": PEN_PERMISSION,
        "declared": bool(block),
        "declaring_package": block.get("package", ""),
        "protectionLevel_pm": block.get("protectionLevel", ""),
    }
    if block.get("package"):
        c, decl, _ = run.adb.shell(["dumpsys", "package", block["package"]], timeout=120)
        run.raw("service", "dumpsys-package-declaring.txt", decl)
        m = re.search(re.escape(PEN_PERMISSION) + r":\s*prot=(\S+)", decl)
        data["permission"]["protectionLevel_dumpsys"] = m.group(1).rstrip(",") if m else ""
    c, xml, _ = run.adb.shell(["grep", "-ril", "safe.IOT", "/system/etc/permissions",
                               "/system_ext/etc/permissions", "/product/etc/permissions",
                               "/vendor/etc/permissions", "/odm/etc/permissions"], quiet=True)
    data["permission"]["allowlist_xml_files"] = sorted(ln.strip() for ln in xml.splitlines() if ln.strip())

    c, svc, _ = run.adb.shell(["dumpsys", "activity", "services", PEN_SERVICE_PKG])
    run.raw("service", "dumpsys-activity-services.txt", svc)
    data["service_running"] = {
        "CoreService_listed": "CoreService" in svc,
        "records": [redact(ln.strip()) for ln in filter_lines(svc, r"ServiceRecord", 20)],
    }

    # Which installed packages look like pen clients, and which hold the permission.
    c, pkgs, _ = run.adb.shell(["pm", "list", "packages"])
    candidates = sorted(ln.split(":", 1)[1].strip() for ln in pkgs.splitlines()
                        if re.search(r"note|ipemanager|pencil|stylus|stylo|\.pen\b|hipaint", ln, re.I))
    holders = {}
    for p in candidates:
        c, d, _ = run.adb.shell(["dumpsys", "package", p])
        m = re.search(re.escape(PEN_PERMISSION) + r": granted=(\w+)", d)
        holders[p] = m.group(1) if m else "not requested"
    data["pen_client_packages"] = holders

    print(f"  {PEN_SERVICE_PKG} {data['versionName'] or '?'} ({data['versionCode'] or '?'}); "
          f"installed={data['installed']}")
    print(f"  {PEN_PERMISSION}: declared by {data['permission']['declaring_package'] or '?'}, "
          f"protectionLevel {data['permission']['protectionLevel_pm'] or '?'} / "
          f"{data['permission'].get('protectionLevel_dumpsys') or '?'}")
    for p, g in holders.items():
        print(f"  {p}: safe.IOT {g}")
    return run.save("service", data)


def step_apk(run):
    """Pull the installed APK(s) of the pen service, record version, size and SHA-256."""
    pkgs = run.packages or [PEN_SERVICE_PKG]
    print(f"apk: pull {', '.join(pkgs)} from the tablet")
    out = []
    for pkg in pkgs:
        c, path, _ = run.adb.shell(["pm", "path", pkg])
        remote = [ln.split(":", 1)[1].strip() for ln in path.splitlines() if ln.startswith("package:")]
        c, dump, _ = run.adb.shell(["dumpsys", "package", pkg])
        version = (re.search(r"versionName=(\S+)", dump) or [None, ""])[1]
        code = (re.search(r"versionCode=(\d+)", dump) or [None, ""])[1]
        entry = {"package": pkg, "installed": bool(remote), "versionName": version,
                 "versionCode": code, "files": []}
        if not remote:
            print(f"  {pkg}: not installed")
            out.append(entry)
            continue
        safe_version = re.sub(r"[^\w.-]", "_", version or "unknown")
        dest_dir = run.out / "raw" / "apk" / pkg / safe_version
        for rpath in remote:
            name = rpath.rsplit("/", 1)[-1]
            local = dest_dir / name
            if local.exists():
                local.unlink()
            pull = run.adb.pull(rpath, local, f"{pkg}-{name}")
            f = {"device_path": rpath, "name": name, "pulled": bool(pull.get("pulled")),
                 "via": pull.get("via") or pull.get("error") or pull.get("note")}
            if pull.get("pulled"):
                f["bytes"] = local.stat().st_size
                f["sha256"] = sha256(local)
                f["local"] = str(local.relative_to(REPO)) if local.is_relative_to(REPO) else str(local)
            entry["files"].append(f)
        out.append(entry)
        print(f"  {pkg} {version} ({code}): {len(entry['files'])} file(s)")
        for f in entry["files"]:
            print(f"    {f['name']}: " + (f"{f['bytes']} bytes, sha256 {f['sha256'][:16]}..."
                                          if f["pulled"] else f"not pulled ({f['via']})"))
    note = ("APKs stay in research/local/ and are never committed. Commit only package, version, "
            "sizes and hashes. Decompiling is limited to the owner-approved interface check "
            "(PICKUP.md Step 2b, research/tools/aidl_check.py).")
    print("  " + note)
    return run.save("apk", {"packages": out, "note": note})


def step_bluetooth(run):
    print("bluetooth: bond and connection state (addresses redacted)")
    c, out, _ = run.adb.shell(["dumpsys", "bluetooth_manager"], timeout=120)
    run.raw("bluetooth", "dumpsys-bluetooth_manager.txt", out)
    pen_rx = r"stylo|stylus|pencil|\bpen\b|oneplus"
    bonded = []
    m = re.search(r"Bonded devices:\n(.*?)(?:\n\s*\n|\Z)", out, re.S)
    if m:
        for ln in m.group(1).splitlines():
            if ln.strip():
                bonded.append(redact(ln.strip()) if re.search(pen_rx, ln, re.I) else "<other device>")
    snoop = run.adb.shell(["getprop", "persist.bluetooth.btsnooplogmode"])[1].strip()
    data = {
        "bonded_pen_like": [b for b in bonded if b != "<other device>"],
        "bonded_other_count": sum(1 for b in bonded if b == "<other device>"),
        "pen_lines": [redact(ln.strip()) for ln in filter_lines(out, pen_rx, 60)],
        "btsnooplogmode": snoop or "(unset)",
    }
    print(f"  pen-like bonds: {len(data['bonded_pen_like'])}; other bonds: {data['bonded_other_count']}; "
          f"HCI snoop mode: {data['btsnooplogmode']}")
    return run.save("bluetooth", data)


def _dmesg(run):
    c, out, err = run.adb.shell(["dmesg"])
    if (c != 0 or not out.strip()) and run.adb.has_root():
        c, out, err = run.adb.shell(["dmesg"], root=True)
        return out, "root"
    return out, ("shell" if out.strip() else f"unavailable ({err.strip()[:80] or 'no output'})")


def step_handshake(run):
    print("handshake: pen attach/detach log")
    run.ask("  Take the pen off the tablet, then press Enter. ")
    c, t0, _ = run.adb.shell(["date", "+%s"])
    run.ask("  Attach the pen, wait 5 s, remove it, attach it again, wait 5 s. Then press Enter. ")
    c, t1, _ = run.adb.shell(["date", "+%s"])
    dmesg, dmesg_via = _dmesg(run)
    run.raw("handshake", "dmesg.txt", dmesg)
    c, logcat, _ = run.adb.shell(["logcat", "-d", "-v", "threadtime"], timeout=120)
    run.raw("handshake", "logcat.txt", logcat)
    k_lines = filter_lines(dmesg, r"cps8601|wireless_pen|cps_wls|firmware.*pen")
    l_lines = filter_lines(logcat, r"ipemanager|cps8601|wireless_pen|Pencil|stylus|btadsorb")
    run.raw("handshake", "dmesg-pen.txt", "\n".join(k_lines) + "\n")
    run.raw("handshake", "logcat-pen.txt", "\n".join(l_lines) + "\n")
    order = []
    for ln in k_lines:
        for mk in HANDSHAKE_MARKERS:
            if mk.lower() in ln.lower():
                order.append(mk)
    data = {
        "window_epoch": [t0.strip(), t1.strip()],
        "dmesg_via": dmesg_via,
        "markers_seen_in_order": order,
        "marker_counts": {mk: order.count(mk) for mk in HANDSHAKE_MARKERS},
        "firmware_load_lines": [redact(ln) for ln in k_lines if re.search(r"firmware|\.bin", ln, re.I)][:40],
        "kernel_excerpt": [redact(ln) for ln in k_lines][-120:],
        "logcat_excerpt": [redact(ln) for ln in l_lines][-120:],
    }
    print(f"  dmesg via {dmesg_via}; kernel lines {len(k_lines)}; logcat lines {len(l_lines)}")
    print(f"  markers: {' -> '.join(order) or 'none'}")
    return run.save("handshake", data)


def step_poc_log(run):
    print(f"poc-log: {POC_LOG_TAG} lines from the Route-A PoC app")
    c, path, _ = run.adb.shell(["pm", "path", POC_PKG])
    c, pkg, _ = run.adb.shell(["dumpsys", "package", POC_PKG])
    version = (re.search(r"versionName=(\S+)", pkg) or [None, ""])[1]
    c, out, _ = run.adb.shell(["logcat", "-d", "-v", "threadtime", "-s", POC_LOG_TAG], timeout=60)
    run.raw("poc-log", "logcat-Stylo2Poc.txt", out)
    lines = [redact(ln) for ln in out.splitlines() if POC_LOG_TAG in ln]
    stages = {s: [ln for ln in lines if re.search(rf"\b{s}\b", ln)] for s in
              ("ENV", "DISCOVERY", "BIND", "AUTHZ", "EXEC")}
    vibrated = run.ask("  Did the pen physically vibrate on '2. Trigger haptic'? [y/n/blank=not run] ").lower()
    data = {
        "installed": "package:" in path,
        "versionName": version,
        "line_count": len(lines),
        "stages": {k: v[-20:] for k, v in stages.items()},
        "physical_result": {"y": "vibrated", "n": "did not vibrate"}.get(vibrated[:1], "not recorded"),
        "note": "Record 'works' only if the pen physically vibrated (android/stylo2-poc/README.md).",
    }
    if not lines:
        print("  no PoC lines in logcat. Run the app (Bind, Trigger, Stop) first.")
    for k, v in stages.items():
        print(f"  {k:9} {len(v)} lines" + (f"  last: {v[-1][-100:]}" if v else ""))
    print(f"  physical result: {data['physical_result']}")
    return run.save("poc-log", data)


# ---------------------------------------------------------------- summary

def step_summary(run):
    print("summary: summary.md")
    S = {s: run.load(s) for s in ("check", "identify", "firmware", "modules", "driver",
                                  "service", "apk", "bluetooth", "handshake", "poc-log")}
    o = ["# Pad 3 read-out summary", "",
         f"Generated {now()} by `research/tools/pad3_readout.py`. Personal identifiers are redacted; "
         "raw output stays in `raw/` and must not be committed.", ""]
    if S["identify"]:
        p = S["identify"]["props"]
        o += ["## Device", "", "| Property | Value |", "| --- | --- |"]
        o += [f"| `{k}` | `{v}` |" for k, v in p.items() if v]
        o += ["", f"Kernel: `{S['identify']['kernel']}`", ""]
        if S["identify"]["region_candidates"]:
            o += ["Region/hardware candidates:", ""]
            o += [f"- `{k}` = `{v}`" for k, v in S["identify"]["region_candidates"].items()]
            o += [""]
    if S["check"] and S["check"]["device"].get("root") is not None:
        o += [f"Root available: **{'yes' if S['check']['device']['root'] else 'no'}**", ""]
    if S["firmware"]:
        f = S["firmware"]
        o += ["## Pen-charger firmware", "",
              f"`{f['firmware_dir']}` folders: {', '.join(f'`{x}`' for x in f['folders']) or 'none'}. "
              f"`ro.boot.prjname` = `{f['prjname']}`, folder present: **{f['prjname_folder_present']}**.", ""]
        if f["symlinks"]:
            o += ["Symlinks: " + ", ".join(f"`{s['name']}` -> `{s['target']}`" for s in f["symlinks"]), ""]
        if f["files"]:
            o += ["| Path | Bytes | Used | SHA-256 | Build |", "| --- | ---: | ---: | --- | --- |"]
            o += [f"| `{x['path']}` | {x.get('bytes', '')} | {x.get('used_bytes', '')} | `{x['sha256']}` "
                  f"| {x['known_build']} |" for x in f["files"]]
            o += [""]
        if f.get("other_cps8601_paths"):
            o += ["Other CPS8601 files: " + ", ".join(f"`{x}`" for x in f["other_cps8601_paths"]), ""]
        if f.get("overlay"):
            o += [f"Overlay `{f['overlay']['path']}` present: {f['overlay']['present']} "
                  f"({', '.join(f['overlay']['folders']) or 'no folders'})", ""]
        o += [f"Pull: {f['pull'].get('via') or 'failed: ' + str(f['pull'].get('error') or f['pull'].get('note'))}", ""]
    if S["modules"]:
        m = S["modules"]
        o += ["## Root modules", ""]
        if m.get("skipped"):
            o += [f"Skipped: {m['skipped']}", ""]
        else:
            o += [f"Root managers: {', '.join(m['root_managers']) or 'none'}", ""]
            o += [f"- `{x['dir']}`: {x['name']} {x['version']} (versionCode {x['versionCode']}) "
                  f"{' '.join(x['flags'])}" + (f"; firmware: {', '.join(x['ships_firmware'])}" if x["ships_firmware"] else "")
                  for x in m["modules"]] or ["- none"]
            o += [""]
    if S["driver"]:
        d = S["driver"]
        o += ["## Wireless-pen driver", "", f"Loaded: {', '.join(d['loaded']) or 'not listed'}. "
              f"File: {', '.join(f'`{p}`' for p in d['paths']) or 'not found'}. Kernel `{d['kernel_release']}`.", ""]
        if d["modinfo"]:
            o += ["```"] + [f"{k}: {v}" for k, v in d["modinfo"].items()] + ["```", ""]
        if d.get("firmware_path_strings"):
            o += ["Firmware-path strings:", ""] + [f"- `{s}`" for s in d["firmware_path_strings"]] + [""]
    if S["service"]:
        s = S["service"]
        pm = s["permission"]
        o += ["## Pen service", "",
              f"`{s['package']}` {s['versionName']} (versionCode {s['versionCode']}), installed {s['installed']}. "
              f"Mentions: {', '.join(k for k, v in s['mentions'].items() if v) or 'none'}.", "",
              f"`{pm['name']}`: declared {pm['declared']} by `{pm['declaring_package']}`, protection level "
              f"`{pm['protectionLevel_pm']}` (pm) / `{pm.get('protectionLevel_dumpsys', '')}` (dumpsys). "
              f"Allowlist XML: {', '.join(pm['allowlist_xml_files']) or 'none found'}.", ""]
        if s["pen_client_packages"]:
            o += [f"- `{k}`: safe.IOT {v}" for k, v in s["pen_client_packages"].items()] + [""]
    if S["bluetooth"]:
        b = S["bluetooth"]
        o += ["## Bluetooth", "", f"Pen-like bonds: {len(b['bonded_pen_like'])}; other bonds: "
              f"{b['bonded_other_count']}; HCI snoop mode: `{b['btsnooplogmode']}`.", ""]
    if S["handshake"]:
        h = S["handshake"]
        o += ["## Attach handshake", "", f"dmesg via {h['dmesg_via']}. Markers in order: "
              f"{' -> '.join(h['markers_seen_in_order']) or 'none'}.", ""]
        if h["firmware_load_lines"]:
            o += ["```"] + h["firmware_load_lines"][:20] + ["```", ""]
    if S["apk"]:
        o += ["## Pen service APK (pulled for the interface check)", "",
              "| Package | Version | File | Bytes | SHA-256 |", "| --- | --- | --- | ---: | --- |"]
        for e in S["apk"]["packages"]:
            if not e["files"]:
                o += [f"| `{e['package']}` | not installed | | | |"]
            for f in e["files"]:
                o += [f"| `{e['package']}` | {e['versionName']} ({e['versionCode']}) | `{f['name']}` "
                      f"| {f.get('bytes', '')} | `{f.get('sha256', 'not pulled')}` |"]
        o += ["", "APKs stay in `research/local/`; never commit them.", ""]
    if S["poc-log"]:
        q = S["poc-log"]
        o += ["## Route-A PoC", "", f"Installed {q['installed']} ({q['versionName']}); {q['line_count']} log lines; "
              f"physical result: **{q['physical_result']}**.", ""]
        for k, v in q["stages"].items():
            if v:
                o += [f"- {k}: `{v[-1][-160:]}`"]
        o += [""]
    o += ["## Where to record this", "",
          "- `research/data/devices.json` -> `dev-oneplus-pad3`",
          "- `research/data/interfaces.json` -> `if-cps8601-firmware`, `if-oplus-wireless-pen-driver`, `if-oplus-ipemanager`",
          "- `research/data/gaps.json` -> `gap-pen-firmware-directory` and the route gaps (see PICKUP.md)",
          "- `research/reference/pad3-firmware/MANIFEST.md` (paths, sizes, hashes; no binaries)",
          "- `research/data/gaps.json` -> `gap-aidl-matches-firmware` (APK version and hash, then `research/tools/aidl_check.py`)",
          "- `stylo2/docs/E-prototype.md` matrix and `stylo2/docs/C-protocol.md` for route results", ""]
    p = run.out / "summary.md"
    p.write_text("\n".join(o), encoding="utf-8")
    print(f"  -> {p}")


STEPS = {
    "check": step_check, "identify": step_identify, "firmware": step_firmware,
    "modules": step_modules, "driver": step_driver, "service": step_service,
    "apk": step_apk, "bluetooth": step_bluetooth, "handshake": step_handshake,
    "poc-log": step_poc_log,
    "summary": step_summary,
}
ALL = ["check", "identify", "firmware", "modules", "driver", "service", "apk", "bluetooth"]


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0],
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("command", choices=list(STEPS) + ["all"])
    ap.add_argument("--adb", default="adb")
    ap.add_argument("-s", dest="serial")
    ap.add_argument("--out", default=str(DEFAULT_OUT))
    ap.add_argument("--root", choices=("auto", "yes", "no"), default="auto")
    ap.add_argument("--no-prompt", action="store_true")
    ap.add_argument("--packages", default="", help="apk step: comma-separated package names")
    a = ap.parse_args(argv)
    run = Run(Adb(a.adb, a.serial, a.root), a.out, prompt=not a.no_prompt)
    run.packages = [p.strip() for p in a.packages.split(",") if p.strip()]
    if a.command not in ("summary", "check"):
        need_device(run.adb)
    for name in (ALL if a.command == "all" else [a.command]):
        STEPS[name](run)
    if a.command not in ("summary", "check"):
        step_summary(run)
    print("Done. Commit only redacted findings (summary.md excerpts, hashes); never research/local/.")


if __name__ == "__main__":
    main()
