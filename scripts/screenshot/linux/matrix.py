#!/usr/bin/env python3
import argparse
import hashlib
import json
import platform
import re
import subprocess
from pathlib import Path

HOSTS = ("egui", "floem", "gpui")
SCENARIOS = {"empty-chat", "standard-chat", "thinking-chat", "narrow-chat", "tall-chat"}
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def arguments():
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--requests", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def load_json(path):
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def validate_baseline(baseline, request_names):
    if request_names != SCENARIOS:
        raise ValueError("all five required scenarios must be present")
    if baseline.get("architecture") != "aarch64":
        raise ValueError("baseline architecture must be aarch64")
    scenarios = baseline.get("scenarios")
    if not isinstance(scenarios, dict) or set(scenarios) != request_names:
        raise ValueError("baseline scenarios must exactly match request files")
    for name, scenario in scenarios.items():
        viewport = scenario.get("viewport", {})
        hashes = scenario.get("sha256", {})
        if set(hashes) != set(HOSTS) or not valid_viewport(viewport):
            raise ValueError(f"invalid baseline scenario: {name}")
        if not all(isinstance(value, str) and SHA256.fullmatch(value) for value in hashes.values()):
            raise ValueError(f"invalid baseline hash: {name}")


def valid_viewport(viewport):
    return (isinstance(viewport, dict) and set(viewport) == {"width", "height"}
            and type(viewport["width"]) is int and type(viewport["height"]) is int
            and 320 <= viewport["width"] <= 4096 and 240 <= viewport["height"] <= 4096)


def validate_requests(paths, baseline):
    requests = {path.stem: load_json(path) for path in paths}
    for name, request in requests.items():
        if request.get("viewport") != baseline["scenarios"][name]["viewport"]:
            raise ValueError(f"viewport differs from baseline: {name}")
    return requests


def run_case(name, host, baseline, output, requests_dir):
    case_dir = output / name / host
    case_dir.mkdir(parents=True, exist_ok=True)
    request = requests_dir / f"{name}.json"
    expected = baseline["scenarios"][name]["sha256"][host]
    command = ["cargo", "run", "--locked", "--manifest-path", "scripts/screenshot/Cargo.toml", "--", "--native-host", host, "--native-request", str(request), "--native-baseline-sha256", expected, "--output", str(case_dir)]
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (case_dir / "capture.log").write_text(result.stdout, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"capture failed: {name}/{host}")
    png = case_dir / f"native-{host}.png"
    return png, expected


def png_dimensions(path):
    data = path.read_bytes()
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise ValueError(f"invalid PNG: {path}")
    return int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")


def verify_png(path, expected_hash, viewport):
    actual_hash = hashlib.sha256(path.read_bytes()).hexdigest()
    dimensions = png_dimensions(path)
    expected_dimensions = (viewport["width"], viewport["height"])
    if actual_hash != expected_hash or dimensions != expected_dimensions:
        raise ValueError(f"baseline mismatch: {path}")
    return {"path": str(path), "sha256": actual_hash, "dimensions": dimensions}


def write_summary(path, metadata):
    path.write_text(json.dumps(metadata, indent=2), encoding="utf-8")


def run_matrix(args, images):
    if platform.system() != "Linux" or platform.machine() not in ("aarch64", "arm64"):
        raise SystemExit("unsupported architecture; expected aarch64")
    baseline = load_json(args.baseline)
    paths = sorted(args.requests.glob("*.json"))
    names = {path.stem for path in paths}
    validate_baseline(baseline, names)
    validate_requests(paths, baseline)
    for name in sorted(names):
        for host in HOSTS:
            png, expected = run_case(name, host, baseline, args.output, args.requests)
            images[f"{name}/{host}"] = verify_png(png, expected, baseline["scenarios"][name]["viewport"])
    return {"status": "passed", "architecture": platform.machine(), "baseline": str(args.baseline), "images": images}


def failed_metadata(args, images, error):
    return {
        "status": "failed",
        "architecture": platform.machine(),
        "baseline": str(args.baseline),
        "images": images,
        "diagnosis": {"type": type(error).__name__, "message": str(error)},
    }


def main():
    args = arguments()
    args.output.mkdir(parents=True, exist_ok=True)
    images = {}
    summary_path = args.output / "summary.json"
    write_summary(summary_path, {"status": "running"})
    try:
        write_summary(summary_path, run_matrix(args, images))
    except BaseException as error:
        write_summary(summary_path, failed_metadata(args, images, error))
        raise


if __name__ == "__main__":
    main()
