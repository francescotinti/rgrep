#!/usr/bin/env python3
"""Export named Criterion estimates and compute deltas from the same medians."""

import argparse
import json
from pathlib import Path


def estimates(root, baseline):
    result = {}
    for path in sorted(root.glob(f"*/*/{baseline}/estimates.json")):
        key = path.parent.parent.relative_to(root).as_posix()
        result[key] = json.loads(path.read_text())
    if not result:
        raise SystemExit(f"No Criterion estimates for {baseline!r} in {root}")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline")
    parser.add_argument("--compare", help="Named pre-change baseline")
    parser.add_argument("--root", type=Path, default=Path("target/criterion"))
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    after = estimates(args.root, args.baseline)
    before = estimates(args.root, args.compare) if args.compare else {}
    if before and before.keys() != after.keys():
        raise SystemExit("Baseline scenarios differ; refusing a partial comparison")
    report = {"baseline": args.baseline, "compare": args.compare, "scenarios": {}}
    print("| Scenario | Pre ms | Post ms | Median delta |")
    print("|---|---:|---:|---:|")
    for name, value in after.items():
        post = value["median"]["point_estimate"]
        pre = before[name]["median"]["point_estimate"] if before else None
        delta = (post / pre - 1) * 100 if pre else None
        report["scenarios"][name] = {
            "pre": before.get(name), "post": value, "median_delta_percent": delta
        }
        pre_text = f"{pre / 1e6:.4f}" if pre else "—"
        delta_text = f"{delta:+.2f}%" if delta is not None else "—"
        print(f"| {name} | {pre_text} | {post / 1e6:.4f} | {delta_text} |")
    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
