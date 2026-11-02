#!/usr/bin/env python3
"""Build combined outputs from research/data/ after validating it.

Writes to research/build/ (not committed):
    stylus-haptics-db.json   every record in one file, with the metadata
    seed/<id>.json           one document per record, without the id, as
                             stored in the database page's "records" collection

Usage:
    python3 research/tools/build.py
"""
import json
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import validate  # noqa: E402

BUILD = Path(__file__).resolve().parent.parent / "build"
TYPE_ORDER = ["source", "device", "interface", "concept", "plan", "gap"]


def main():
    meta, records, errors, _ = validate.validate()
    if errors:
        for e in errors:
            print("ERROR", e)
        sys.exit("Fix the errors above before building.")
    clean = [{k: v for k, v in r.items() if not k.startswith("_")} for r in records]
    clean.sort(key=lambda r: (TYPE_ORDER.index(r["type"]), r["id"]))

    if BUILD.exists():
        shutil.rmtree(BUILD)
    (BUILD / "seed").mkdir(parents=True)
    combined = {k: v for k, v in meta.items() if k != "files"}
    combined["records"] = clean
    (BUILD / "stylus-haptics-db.json").write_text(
        json.dumps(combined, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for r in clean:
        doc = {k: v for k, v in r.items() if k != "id"}
        (BUILD / "seed" / f"{r['id']}.json").write_text(
            json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"Built {len(clean)} records into {BUILD}")


if __name__ == "__main__":
    main()
