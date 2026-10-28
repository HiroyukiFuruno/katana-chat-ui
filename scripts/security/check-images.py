#!/usr/bin/env python3
"""Issue #13のLinux/arm64 imageをID固定で検査し、所有containerの終了を確認する。"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PROBE = ROOT / "scripts/security/check-container-tls.py"
OWNER_LABEL = "com.katana.kcu.issue13.check-owner"
RUN_LABEL = "com.katana.kcu.issue13.check-run"
EXPECTED_HOSTS = {"control", "g3", "aaa", "noct", "aaa_noct"}
CID_PATTERN = re.compile(r"[0-9a-f]{64}")
ACTIVE_DOCKER_PROCESS: subprocess.Popen[str] | None = None
SHUTDOWN_SIGNAL: int | None = None
CLEANUP_ACTIVE = False
SOURCE_FILES = (
    PROBE,
    Path(__file__).resolve(),
    ROOT / "scripts/screenshot/linux/Dockerfile",
    ROOT / "scripts/manual-ui/ime/Dockerfile",
)
SHELL = r"""set +e
python3 -B /check-container-tls.py --image-id "$1" --output /tmp/result.json > /tmp/summary.json 2> /tmp/probe.stderr.log
probe_rc=$?
if [ -f /tmp/result.json ]; then
  python3 -B -c 'import json,sys; p=sys.argv[2]; d=json.load(open(p)); d["probe_actual_exit"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,sort_keys=True)+"\n")' "$probe_rc" /tmp/result.json
else
  python3 -B -c 'import json,sys; print(json.dumps({"image_id":sys.argv[2],"aggregate_acceptance":"UNJUDGED","probe_actual_exit":int(sys.argv[1]),"hosts":{},"error":"probe did not produce result.json"},sort_keys=True))' "$probe_rc" "$1" > /tmp/result.json
fi
if [ -f /tmp/summary.json ]; then cat /tmp/summary.json >&2; fi
if [ -f /tmp/probe.stderr.log ]; then cat /tmp/probe.stderr.log >&2; fi
cat /tmp/result.json
exit "$probe_rc"
"""


def runtime_prefix(rtk_path: str | None) -> list[str]:
    """RTKがあれば絶対pathとproxyを使い、なければDocker CLIを選ぶ。"""
    if not rtk_path:
        return ["docker"]
    return [str(Path(rtk_path).absolute()), "proxy", "docker"]


DOCKER_CMD = runtime_prefix(shutil.which("rtk"))


def command(argv: list[str], timeout: int | None = None) -> subprocess.CompletedProcess[str]:
    global ACTIVE_DOCKER_PROCESS
    process = subprocess.Popen(argv, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    ACTIVE_DOCKER_PROCESS = process
    try:
        if SHUTDOWN_SIGNAL is not None and not CLEANUP_ACTIVE and process.poll() is None:
            try:
                process.terminate()
            except OSError:
                pass
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired as error:
            try:
                process.kill()
            except ProcessLookupError:
                pass
            stdout, stderr = process.communicate()
            raise subprocess.TimeoutExpired(argv, timeout, output=stdout, stderr=stderr) from error
        except KeyboardInterrupt:
            try:
                process.terminate()
            except ProcessLookupError:
                pass
            try:
                process.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                try:
                    process.kill()
                except ProcessLookupError:
                    pass
                process.communicate()
            raise
        return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)
    finally:
        ACTIVE_DOCKER_PROCESS = None


def _git_environment(environ: dict[str, str] | None = None) -> dict[str, str]:
    source = os.environ if environ is None else environ
    sanitized = {key: value for key, value in source.items() if not key.startswith("GIT_")}
    sanitized["GIT_OPTIONAL_LOCKS"] = "0"
    sanitized["LC_ALL"] = "C"
    return sanitized


def _source_identity() -> dict[str, Any]:
    env = _git_environment()

    def git_output(*arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(ROOT), *arguments],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            env=env,
        )
        if result.returncode != 0:
            raise RuntimeError(f"git {arguments[0]} failed with exit {result.returncode}")
        return result.stdout

    head = git_output("rev-parse", "HEAD").strip()
    status = git_output("status", "--porcelain=v1", "--untracked-files=all", "--no-renames")
    hashes: dict[str, str] = {}
    for path in SOURCE_FILES:
        if not path.is_file():
            raise RuntimeError(f"source identity file is missing: {path}")
        hashes[str(path.relative_to(ROOT))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return {
        "scope": "inspection checkout at invocation time; does not prove which source built an image",
        "source_head": head,
        "source_dirty": bool(status),
        "changed_entry_count": sum(1 for line in status.splitlines() if line),
        "status_sha256": hashlib.sha256(status.encode("utf-8")).hexdigest(),
        "source_sha256": hashes,
    }


def _handle_sigterm(signum: int, _frame: Any) -> None:
    """probe中はCLIを止め、呼出し側にCID cleanupとreceipt保存を続けさせる。"""
    global SHUTDOWN_SIGNAL
    SHUTDOWN_SIGNAL = signum
    process = ACTIVE_DOCKER_PROCESS
    if not CLEANUP_ACTIVE and process is not None and process.poll() is None:
        try:
            process.terminate()
        except OSError:
            pass


def cleanup_command(argv: list[str], timeout: int = 30) -> subprocess.CompletedProcess[str]:
    """cleanup中のtimeoutやSIGINTを未判定exitとして記録できる形にする。"""
    try:
        return command(argv, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout.decode("utf-8", "replace") if isinstance(error.stdout, bytes) else (error.stdout or "")
        stderr = error.stderr.decode("utf-8", "replace") if isinstance(error.stderr, bytes) else (error.stderr or "docker cleanup command timed out")
        return subprocess.CompletedProcess(argv, 124, stdout, stderr)
    except KeyboardInterrupt:
        return subprocess.CompletedProcess(argv, 130, "", "docker cleanup command interrupted by SIGINT")


def save(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")


def container_is_absent(returncode: int, stdout: str, cid: str) -> bool:
    """成功した完全ID照会で空出力の場合だけ消滅と判定する。"""
    if returncode != 0:
        return False
    rows = [line.strip() for line in stdout.splitlines() if line.strip()]
    return not rows and bool(CID_PATTERN.fullmatch(cid))


def sanitized_image_record(image: dict[str, Any]) -> dict[str, Any]:
    """image inspectから必要項目だけを保存し、他の環境変数を落とす。"""
    env = image.get("Config", {}).get("Env") or []
    ssl_names = sorted({item.split("=", 1)[0] for item in env
                        if isinstance(item, str) and item.startswith(("SSL_CERT_FILE=", "SSL_CERT_DIR="))})
    return {
        "Id": image.get("Id"),
        "Os": image.get("Os"),
        "Architecture": image.get("Architecture"),
        "RepoTags": image.get("RepoTags") or [],
        "RootFS": image.get("RootFS"),
        "ssl_env_override_names": ssl_names,
    }


def owned_container_matches(
    inspect_records: Any,
    cid: str,
    image_id: str,
    owner_token: str,
    run_token: str,
) -> tuple[bool, dict[str, Any]]:
    """stop前にCID・immutable image・両owner labelを照合する。"""
    if not isinstance(inspect_records, list) or len(inspect_records) != 1:
        return False, {"inspect_record_count": len(inspect_records) if isinstance(inspect_records, list) else None}
    record = inspect_records[0]
    labels = record.get("Config", {}).get("Labels") or {}
    safe = {
        "Id": record.get("Id"),
        "Image": record.get("Image"),
        "running": (record.get("State") or {}).get("Running"),
        "owner_label_matches": labels.get(OWNER_LABEL) == owner_token,
        "run_label_matches": labels.get(RUN_LABEL) == run_token,
    }
    matches = (record.get("Id") == cid and record.get("Image") == image_id
               and safe["owner_label_matches"] and safe["run_label_matches"])
    return matches, safe


def verify_and_cleanup_container(
    item_dir: Path,
    cid: str | None,
    image_id: str,
    owner_token: str,
    run_token: str,
) -> dict[str, Any]:
    """完全IDで照会し、自分の一致containerだけ通常stopで回収する。"""
    record: dict[str, Any] = {"owned_container_gone": False, "status": "UNJUDGED", "queries": []}
    if not cid or not CID_PATTERN.fullmatch(cid):
        record["detail"] = "CID file is missing or is not a 64-character hexadecimal ID"
        return record
    record["container_id"] = cid
    query_argv = DOCKER_CMD + ["ps", "--all", "--quiet", "--no-trunc", "--filter", f"id={cid}"]
    for attempt in range(1, 11):
        query = cleanup_command(query_argv, timeout=30)
        save(item_dir / f"cleanup-query-{attempt:02d}.stdout.txt", query.stdout)
        save(item_dir / f"cleanup-query-{attempt:02d}.stderr.log", query.stderr)
        save(item_dir / f"cleanup-query-{attempt:02d}.exit", f"{query.returncode}\n")
        query_info = {"attempt": attempt, "exit": query.returncode,
                      "stdout_lines": [line for line in query.stdout.splitlines() if line.strip()]}
        record["queries"].append(query_info)
        if query.returncode != 0:
            record["detail"] = "docker ps exact-ID query failed; absence is unproven"
            return record
        if container_is_absent(query.returncode, query.stdout, cid):
            record.update({"owned_container_gone": True, "status": "PASS", "detail": "exact-ID query succeeded with empty output"})
            return record
        rows = [line.strip() for line in query.stdout.splitlines() if line.strip()]
        if rows != [cid]:
            record["detail"] = "exact-ID query returned an unexpected container ID"
            return record

        inspect = cleanup_command(DOCKER_CMD + ["container", "inspect", cid], timeout=30)
        save(item_dir / f"cleanup-inspect-{attempt:02d}.stderr.log", inspect.stderr)
        save(item_dir / f"cleanup-inspect-{attempt:02d}.exit", f"{inspect.returncode}\n")
        matches = False
        safe_inspect: dict[str, Any] = {}
        if inspect.returncode == 0:
            try:
                matches, safe_inspect = owned_container_matches(
                    json.loads(inspect.stdout), cid, image_id, owner_token, run_token,
                )
            except json.JSONDecodeError:
                safe_inspect = {"error": "container inspect JSON is invalid"}
        record.setdefault("cleanup_checks", []).append({"inspect_exit": inspect.returncode, **safe_inspect})
        if inspect.returncode != 0:
            record["detail"] = "container inspect failed; exact-ID absence will be checked again"
            time.sleep(1)
            continue
        if not matches:
            record["detail"] = "container identity did not match this run; it was not stopped"
            return record
        if not safe_inspect.get("running"):
            record["detail"] = "owned container has stopped; exact-ID absence will be checked again"
            time.sleep(1)
            continue

        stop = cleanup_command(DOCKER_CMD + ["stop", "--time", "10", cid], timeout=30)
        save(item_dir / f"cleanup-stop-{attempt:02d}.stdout.txt", stop.stdout)
        save(item_dir / f"cleanup-stop-{attempt:02d}.stderr.log", stop.stderr)
        save(item_dir / f"cleanup-stop-{attempt:02d}.exit", f"{stop.returncode}\n")
        record.setdefault("stop_results", []).append({"exit": stop.returncode, "force": False})
        if stop.returncode != 0:
            record["detail"] = "normal docker stop failed; exact-ID absence will still be checked"
        time.sleep(1)
    record["detail"] = "owned container did not disappear after bounded exact-ID checks"
    return record


def main() -> int:
    global SHUTDOWN_SIGNAL
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, type=Path,
                        help="fresh output directory under repository tmp/")
    parser.add_argument("images", nargs="+", help="one or two local image tags or IDs")
    args = parser.parse_args()
    if not 1 <= len(args.images) <= 2:
        parser.error("pass one or two images")
    output = args.output_dir.expanduser().resolve()
    try:
        output.relative_to((ROOT / "tmp").resolve())
    except ValueError:
        parser.error("--output-dir must be under repository tmp/")
    if output.exists():
        parser.error("--output-dir must be fresh and must not already exist")
    if not PROBE.is_file():
        parser.error(f"probe file is missing: {PROBE}")
    SHUTDOWN_SIGNAL = None
    signal.signal(signal.SIGTERM, _handle_sigterm)
    source_identity = _source_identity()
    output.mkdir(parents=True, exist_ok=False)
    probe_sha = source_identity["source_sha256"][str(PROBE.relative_to(ROOT))]
    owner_token = uuid.uuid4().hex
    run_id = uuid.uuid4().hex
    save(output / "invocation.json", json.dumps({
        "image_refs": args.images,
        "probe_path": str(PROBE),
        "probe_sha256": probe_sha,
        "source_checkout_identity": source_identity,
        "docker_cli_prefix": DOCKER_CMD,
        "container_policy": ["--pull=never", "--platform linux/arm64", "--network default",
                             "--read-only", "--tmpfs /tmp", "--cap-drop ALL",
                             "--entrypoint /bin/sh", "--rm", "one read-only probe-file mount"],
        "ssl_environment_overrides": "none",
        "owner_token_sha256": hashlib.sha256(owner_token.encode()).hexdigest(),
        "run_id": run_id,
    }, indent=2) + "\n")

    results: list[dict[str, Any]] = []
    all_pass = True
    for index, image_ref in enumerate(args.images, start=1):
        if SHUTDOWN_SIGNAL is not None:
            all_pass = False
            break
        slug = re.sub(r"[^A-Za-z0-9_.-]+", "-", image_ref).strip("-.")[:48] or "image"
        item_dir = output / f"{index}-{slug}"
        item_dir.mkdir()
        inspected = cleanup_command(DOCKER_CMD + ["image", "inspect", image_ref], timeout=30)
        item: dict[str, Any] = {"requested_ref": image_ref, "directory": item_dir.name,
                                "inspect_exit": inspected.returncode}
        results.append(item)
        if inspected.returncode != 0:
            save(item_dir / "image-inspect.stderr.log", inspected.stderr)
            save(item_dir / "image-inspect.exit", f"{inspected.returncode}\n")
            item.update({"aggregate_acceptance": "UNJUDGED", "error": "local image inspect failed"})
            all_pass = False
            continue
        try:
            records = json.loads(inspected.stdout)
            if len(records) != 1:
                raise ValueError("inspect did not return exactly one image")
            image = records[0]
        except (json.JSONDecodeError, ValueError) as error:
            save(item_dir / "image-inspect.stderr.log", inspected.stderr)
            save(item_dir / "image-inspect.exit", f"{inspected.returncode}\n")
            item.update({"aggregate_acceptance": "UNJUDGED", "error": f"invalid inspect JSON: {error}"})
            all_pass = False
            continue

        safe_image = sanitized_image_record(image)
        save(item_dir / "image-inspect.json", json.dumps(safe_image, indent=2, sort_keys=True) + "\n")
        save(item_dir / "image-inspect.stderr.log", inspected.stderr)
        save(item_dir / "image-inspect.exit", f"{inspected.returncode}\n")
        image_id = safe_image.get("Id") or ""
        ssl_env = safe_image["ssl_env_override_names"]
        item.update({"image_id": image_id, "os": safe_image.get("Os"),
                     "architecture": safe_image.get("Architecture"),
                     "repo_tags": safe_image.get("RepoTags") or [],
                     "ssl_env_override_names": ssl_env})
        if safe_image.get("Os") != "linux" or safe_image.get("Architecture") != "arm64" or not re.fullmatch(r"sha256:[0-9a-f]{64}", image_id):
            item.update({"aggregate_acceptance": "UNJUDGED", "error": "not a valid immutable linux/arm64 image"})
            all_pass = False
            continue
        if ssl_env:
            item.update({"aggregate_acceptance": "UNJUDGED", "error": "image defines an SSL_CERT_FILE/SSL_CERT_DIR override"})
            all_pass = False
            continue

        cidfile = item_dir / "container.cid"
        if cidfile.exists():
            item.update({"aggregate_acceptance": "UNJUDGED", "error": "fresh CID file already exists"})
            all_pass = False
            continue
        owner_run_label = f"{run_id}-{index}"
        run_argv = DOCKER_CMD + ["run", "--pull=never", "--rm", "--platform", "linux/arm64",
            "--network", "default", "--read-only", "--tmpfs", "/tmp:rw,noexec,nosuid,size=64m",
            "--cap-drop", "ALL", "--security-opt", "no-new-privileges", "--cpus", "1",
            "--label", f"{OWNER_LABEL}={owner_token}", "--label", f"{RUN_LABEL}={owner_run_label}",
            "--cidfile", str(cidfile), "--mount",
            f"type=bind,src={PROBE},dst=/check-container-tls.py,readonly",
            "--entrypoint", "/bin/sh", image_id, "-c", SHELL, "kcu-tls-probe", image_id]
        item["docker_run_argv"] = run_argv
        cli_exit = 128 + SHUTDOWN_SIGNAL if SHUTDOWN_SIGNAL is not None else 1
        stdout = ""
        stderr = "probe skipped because shutdown was requested" if SHUTDOWN_SIGNAL is not None else ""
        probe_result: dict[str, Any] = {
            "image_id": image_id,
            "aggregate_acceptance": "UNJUDGED",
            "hosts": {},
            "error": "probe did not produce result.json",
        }
        cleanup: dict[str, Any] = {"owned_container_gone": False, "status": "UNJUDGED"}
        try:
            if SHUTDOWN_SIGNAL is None:
                try:
                    run = command(run_argv, timeout=180)
                    cli_exit = run.returncode
                    stdout, stderr = run.stdout, run.stderr
                except subprocess.TimeoutExpired as error:
                    cli_exit = 124
                    stdout = error.stdout.decode("utf-8", "replace") if isinstance(error.stdout, bytes) else (error.stdout or "")
                    stderr = error.stderr.decode("utf-8", "replace") if isinstance(error.stderr, bytes) else (error.stderr or "container probe command timed out")
                except KeyboardInterrupt:
                    cli_exit = 130
                    stdout, stderr = "", "container probe interrupted by SIGINT"
            save(item_dir / "container.stdout.json", stdout)
            save(item_dir / "container.stderr.log", stderr)
            save(item_dir / "container.exit", f"{cli_exit}\n")
            item["container_exit"] = cli_exit
            try:
                loaded = json.loads(stdout)
                if not isinstance(loaded, dict):
                    raise ValueError("stdout is not a JSON object")
                probe_result = loaded
            except (json.JSONDecodeError, ValueError) as error:
                probe_result["error"] = f"container did not return probe JSON: {error}"
            item["result"] = probe_result
            save(item_dir / "result.json", json.dumps(probe_result, indent=2, sort_keys=True) + "\n")
        finally:
            try:
                cid = cidfile.read_text(encoding="utf-8").strip() if cidfile.is_file() else None
            except OSError as error:
                cid = None
                item["cid_file_error"] = f"{type(error).__name__}: {error}"
            global CLEANUP_ACTIVE
            CLEANUP_ACTIVE = True
            try:
                cleanup = verify_and_cleanup_container(item_dir, cid, image_id, owner_token, owner_run_label)
            finally:
                CLEANUP_ACTIVE = False
            item.update(cleanup)

        item["container_exit"] = cli_exit
        item["shutdown_signal"] = SHUTDOWN_SIGNAL
        hosts = probe_result.get("hosts", {})
        hosts_pass = (isinstance(hosts, dict) and set(hosts) == EXPECTED_HOSTS
                      and all(isinstance(value, dict) and value.get("acceptance") == "PASS" for value in hosts.values()))
        passed = (cli_exit == 0 and probe_result.get("probe_actual_exit") == 0
                  and probe_result.get("image_id") == image_id
                  and probe_result.get("aggregate_acceptance") == "PASS"
                  and hosts_pass and cleanup["owned_container_gone"])
        item["all_five_host_acceptance_pass"] = hosts_pass
        item["wrapper_image_pass"] = passed
        if not passed or SHUTDOWN_SIGNAL is not None:
            all_pass = False
        if SHUTDOWN_SIGNAL is not None:
            break

    if SHUTDOWN_SIGNAL is not None:
        all_pass = False
    receipt = {"schema_version": 3, "probe_sha256": probe_sha,
               "source_checkout_identity": source_identity,
               "ssl_environment_overrides": "none", "results": results,
               "shutdown_signal": SHUTDOWN_SIGNAL,
               "all_images_passed": all_pass}
    payload = json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    save(output / "receipt.json", payload)
    sys.stdout.write(payload)
    return 0 if all_pass and SHUTDOWN_SIGNAL is None else 1


if __name__ == "__main__":
    raise SystemExit(main())
