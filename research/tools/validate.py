#!/usr/bin/env python3
"""Validate the research database in research/data/.

Checks every record against the rules in research/schema/record.schema.json,
plus cross-record rules: unique IDs, IDs prefixed by type, records stored in the
file for their type, and related IDs that exist.

Usage:
    python3 research/tools/validate.py            # report errors, exit 1 if any
    python3 research/tools/validate.py --verbose  # also list one-way relations

Uses only the Python standard library.
"""
import json
import re
import sys
from pathlib import Path

DATA = Path(__file__).resolve().parent.parent / "data"

FIELDS = ["id", "type", "title", "vendor", "category", "status", "priority", "url", "links",
          "summary", "details", "tags", "related", "basis", "updated"]
TYPES = {"source": ("src-", "repo-"), "device": ("dev-",), "interface": ("if-",),
         "concept": ("c-",), "plan": ("plan-",), "gap": ("gap-",)}
STATUSES = {"verified", "unverified", "inferred", "open"}
PRIORITIES = {"critical", "high", "medium", "low"}
ID_RE = re.compile(r"^(src|repo|dev|if|c|plan|gap)-[a-z0-9]+(-[a-z0-9]+)*$")
TAG_RE = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
URL_RE = re.compile(r"^https?://\S+$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def load():
    """Return (meta, records, errors). Each record gets a private _file key."""
    errors = []
    meta = json.loads((DATA / "meta.json").read_text(encoding="utf-8"))
    files = meta.get("files", {})
    if set(files) != set(TYPES):
        errors.append(f"meta.json: 'files' must map exactly these types: {sorted(TYPES)}")
    records = []
    for rtype, name in files.items():
        path = DATA / name
        doc = json.loads(path.read_text(encoding="utf-8"))
        if doc.get("type") != rtype:
            errors.append(f"{name}: top-level type is {doc.get('type')!r}, expected {rtype!r}")
        for r in doc.get("records", []):
            r = dict(r)
            r["_file"] = name
            r["_expected_type"] = rtype
            records.append(r)
    return meta, records, errors


def check_record(r):
    errs = []
    rid = r.get("id", "<no id>")
    where = f"{r['_file']}: {rid}"
    keys = [k for k in r if not k.startswith("_")]
    missing = [k for k in FIELDS if k not in r]
    extra = [k for k in keys if k not in FIELDS]
    if missing:
        errs.append(f"{where}: missing {missing}")
    if extra:
        errs.append(f"{where}: unknown fields {extra}")
    if missing:
        return errs

    def is_str(k, allow_empty=False):
        v = r[k]
        if not isinstance(v, str) or (not allow_empty and not v.strip()):
            errs.append(f"{where}: {k} must be a non-empty string")
            return False
        return True

    if not ID_RE.match(rid) or len(rid) > 80:
        errs.append(f"{where}: id must be a lowercase slug with a type prefix")
    if r["type"] not in TYPES:
        errs.append(f"{where}: unknown type {r['type']!r}")
    else:
        if r["type"] != r["_expected_type"]:
            errs.append(f"{where}: type {r['type']} stored in {r['_file']}")
        if not rid.startswith(TYPES[r["type"]]):
            errs.append(f"{where}: id prefix should be one of {TYPES[r['type']]}")
    for k in ("title", "vendor", "summary", "basis"):
        is_str(k)
    is_str("category", allow_empty=True)
    if r["status"] not in STATUSES:
        errs.append(f"{where}: status {r['status']!r} not in {sorted(STATUSES)}")
    if r["priority"] not in PRIORITIES:
        errs.append(f"{where}: priority {r['priority']!r} not in {sorted(PRIORITIES)}")
    if not isinstance(r["url"], str) or (r["url"] and not URL_RE.match(r["url"])):
        errs.append(f"{where}: url must be empty or start with http(s)://")
    if not isinstance(r["links"], list):
        errs.append(f"{where}: links must be a list")
    else:
        for i, link in enumerate(r["links"]):
            if not isinstance(link, dict) or set(link) != {"label", "url"} \
               or not isinstance(link["label"], str) or not URL_RE.match(str(link["url"])):
                errs.append(f"{where}: links[{i}] must be {{label, url}} with an http(s) url")
    for k in ("details", "tags", "related"):
        v = r[k]
        if not isinstance(v, list) or not all(isinstance(x, str) and x.strip() for x in v):
            errs.append(f"{where}: {k} must be a list of non-empty strings")
        elif k != "details" and len(set(v)) != len(v):
            errs.append(f"{where}: {k} has duplicates")
    if isinstance(r["tags"], list):
        bad = [t for t in r["tags"] if isinstance(t, str) and not TAG_RE.match(t)]
        if bad:
            errs.append(f"{where}: tags must be lowercase-kebab: {bad}")
    if not isinstance(r["updated"], str) or not DATE_RE.match(r["updated"]):
        errs.append(f"{where}: updated must be YYYY-MM-DD")
    return errs


def validate(verbose=False):
    meta, records, errors = load()
    seen = {}
    for r in records:
        errors.extend(check_record(r))
        rid = r.get("id")
        if rid in seen:
            errors.append(f"duplicate id {rid} in {seen[rid]} and {r['_file']}")
        seen[rid] = r["_file"]
    ids = set(seen)
    one_way = []
    by_id = {r["id"]: r for r in records if "id" in r}
    for r in records:
        for rel in r.get("related", []) if isinstance(r.get("related"), list) else []:
            if rel == r["id"]:
                errors.append(f"{r['_file']}: {r['id']}: relates to itself")
            elif rel not in ids:
                errors.append(f"{r['_file']}: {r['id']}: related id not found: {rel}")
            elif r["id"] not in by_id[rel].get("related", []):
                one_way.append((r["id"], rel))
    return meta, records, errors, one_way


def main():
    verbose = "--verbose" in sys.argv
    meta, records, errors, one_way = validate(verbose)
    for e in errors:
        print("ERROR", e)
    counts = {}
    for r in records:
        counts[r.get("type")] = counts.get(r.get("type"), 0) + 1
    status = {}
    for r in records:
        status[r.get("status")] = status.get(r.get("status"), 0) + 1
    print(f"{len(records)} records", counts)
    print("status", status)
    print(f"{len(one_way)} one-way relations (allowed)")
    if verbose:
        for a, b in one_way:
            print(f"  {a} -> {b}")
    if errors:
        print(f"{len(errors)} error(s)")
        sys.exit(1)
    print("OK")


if __name__ == "__main__":
    main()
