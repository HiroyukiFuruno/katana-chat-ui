#!/usr/bin/env python3
"""KCU Issue #13 の既定CAストアと5ホストTLS受入を読み取り検査する。"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import socket
import ssl
import subprocess
import urllib.error
import urllib.request
from pathlib import Path
from typing import Mapping

HOSTS = {
    "control": "https://emudhra.we-have-a-problem.com/",
    "g3": "https://dv-g3b-emudhra.we-have-a-problem.com/",
    "aaa": "https://dv-g3b-aaa-emudhra.we-have-a-problem.com/",
    "noct": "https://dv-g3b-noct-emudhra.we-have-a-problem.com/",
    "aaa_noct": "https://dv-g3b-aaa-noct-emudhra.we-have-a-problem.com/",
}
TIMEOUT_SECONDS = 20
IMPACTED_INTERMEDIATES_SOURCE = "https://emudhra.we-have-a-problem.com/impacted-intermediates.json"
IMPACTED_INTERMEDIATES_SOURCE_SNAPSHOT_SHA256 = "f67b3abe9b4879f4e91874ed041beccd71257f1db41c588c724ccf12aa90c59e"
IMPACTED_INTERMEDIATE_SPKI_SHA256 = frozenset({
    "43bffa19d6ef651451f6841e019931e96f6f2eaeff28ab4b6f1ce4576de2717b",
    "49dd353dc2b919a3770645117d0e60c16115e7d1bc259eefe734b00ea434fe5f",
    "5c0660a2b164910d8ff589c18e14552e9e0ec237dcefe2cb736d41a4811f81a0",
    "63ecefee431d49523bc1136c4e72d0206a477086caf606cd5cf025686c109cf8",
    "92c063d67e8f14db71d288e11266ffdaf69cbf115f7bab14f3bc2f68d25e0e30",
    "e0254c7fcdc265e2f0f8a428dabb75ab4ae039b4872d62f5ee57f0877d5a8a41",
    "ebed6475c2c2d80fcb199f3fe8dd6e6b3ce60b77eb15d4e42c3dec0c6f5b5105",
    "ec4ffa1ad0ac88081314b7aaffef6a91f97cd57291bb5ba686c5415f5eb4f6b6",
})
FORBIDDEN_TRUST_ANCHOR_DER_SHA256 = frozenset({
    "86a1ecba089c4a8d3bbe2734c612ba341d813e043cf9e8a862cd5c57a36bbe6b",
    "d7a7a0fb5d7e2731d771e9484ebcdef71d5f0c3e0a2948782bc83ee0ea699ef4",
})
OBSERVED_ISSUER_FAILURE_VERIFY_CODES = frozenset({20})


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def certificate_public_key(der: bytes, index: int) -> tuple[bytes, str, str]:
    pem = b"-----BEGIN CERTIFICATE-----\n" + base64.b64encode(der) + b"\n-----END CERTIFICATE-----\n"
    output = subprocess.run(
        ["openssl", "x509", "-noout", "-subject", "-issuer", "-pubkey"],
        input=pem, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
    ).stdout
    pubkey_start = output.find(b"-----BEGIN PUBLIC KEY-----")
    if pubkey_start < 0:
        raise RuntimeError(f"CA {index} の公開鍵をopensslから取得できません")
    pubkey_der = subprocess.run(
        ["openssl", "pkey", "-pubin", "-outform", "DER"],
        input=output[pubkey_start:], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
    ).stdout
    lines = output[:pubkey_start].decode("utf-8", "replace").splitlines()
    subject = next((line.strip() for line in lines if line.startswith("subject=")), "")
    issuer = next((line.strip() for line in lines if line.startswith("issuer=")), "")
    return pubkey_der, subject, issuer


def cert_inventory(ctx: ssl.SSLContext) -> list[dict[str, str]]:
    result = []
    for index, der in enumerate(ctx.get_ca_certs(binary_form=True)):
        pubkey_der, subject, issuer = certificate_public_key(der, index)
        result.append({
            "subject": subject,
            "issuer": issuer,
            "der_sha256": hashlib.sha256(der).hexdigest(),
            "spki_sha256": hashlib.sha256(pubkey_der).hexdigest(),
        })
    return result


def canonical_inventory_sha256(inventory: list[dict[str, str]]) -> str:
    """標準TLS contextのCA一覧を順序に依存しないJSONとしてhashする。"""
    ordered = sorted(inventory, key=lambda item: (item["der_sha256"], item["spki_sha256"]))
    encoded = json.dumps(ordered, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def native_cafile_sha256(cafile: str | None) -> dict[str, str | None]:
    """Python/OpenSSLが選んだcafileの実バイト列をhashする。"""
    if not cafile:
        return {"status": "UNJUDGED", "path": cafile, "sha256": None, "detail": "default cafile is unavailable"}
    path = Path(cafile)
    try:
        if not path.is_file():
            return {"status": "UNJUDGED", "path": str(path), "sha256": None, "detail": "default cafile is not a regular file"}
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        return {"status": "UNJUDGED", "path": str(path), "sha256": None, "detail": f"{type(error).__name__}: {error}"}
    return {"status": "PASS", "path": str(path), "sha256": digest, "detail": "raw cafile bytes hashed"}


def forbidden_inventory_matches(inventory: list[dict[str, str]]) -> dict[str, object]:
    der_matches = sorted({item["der_sha256"].lower() for item in inventory} & FORBIDDEN_TRUST_ANCHOR_DER_SHA256)
    inventory_spki = {item["spki_sha256"].lower() for item in inventory}
    spki_matches = sorted(inventory_spki & IMPACTED_INTERMEDIATE_SPKI_SHA256)
    return {"distrusted_anchor_der_sha256_present": der_matches,
            "impacted_intermediate_spki_sha256_present": spki_matches,
            "impacted_intermediate_spki_sha256_absent": sorted(IMPACTED_INTERMEDIATE_SPKI_SHA256 - inventory_spki),
            "impacted_intermediate_spki_expected_count": len(IMPACTED_INTERMEDIATE_SPKI_SHA256)}


def _verification_details(error: BaseException) -> tuple[int | None, str | None]:
    code = getattr(error, "verify_code", None)
    message = getattr(error, "verify_message", None)
    return (code if type(code) is int else None, message if isinstance(message, str) else None)


def classify_error(error: BaseException) -> tuple[str, str, int | None, str | None]:
    if isinstance(error, ssl.SSLCertVerificationError):
        code, message = _verification_details(error)
        return "tls_certificate_verification_error", str(error), code, message
    if isinstance(error, urllib.error.HTTPError):
        return "http_response", f"HTTP {error.code} {error.reason}", None, None
    if isinstance(error, urllib.error.URLError):
        reason = error.reason
        if isinstance(reason, ssl.SSLCertVerificationError):
            code, message = _verification_details(reason)
            return "tls_certificate_verification_error", str(reason), code, message
        if isinstance(reason, (socket.timeout, TimeoutError)):
            return "timeout", str(reason), None, None
        if isinstance(reason, socket.gaierror):
            return "dns_error", str(reason), None, None
        if isinstance(reason, OSError):
            return "connection_error", str(reason), None, None
        return "url_error", repr(reason), None, None
    if isinstance(error, (socket.timeout, TimeoutError)):
        return "timeout", str(error), None, None
    if isinstance(error, socket.gaierror):
        return "dns_error", str(error), None, None
    if isinstance(error, OSError):
        return "connection_error", str(error), None, None
    return "unexpected_error", f"{type(error).__name__}: {error}", None, None


def host_acceptances(host_results: Mapping[str, Mapping[str, object]]) -> dict[str, str]:
    """DNS/timeout/接続失敗をTLS証明書拒否として扱わず受入状態を返す。"""
    acceptances = {}
    for key in HOSTS:
        result = host_results.get(key)
        kind = result.get("classification") if isinstance(result, Mapping) else None
        verify_code = result.get("verify_code") if isinstance(result, Mapping) else None
        verify_message = result.get("verify_message") if isinstance(result, Mapping) else None
        trusted_chain_rejection = (
            kind == "tls_certificate_verification_error"
            and type(verify_code) is int
            and verify_code in OBSERVED_ISSUER_FAILURE_VERIFY_CODES
            and isinstance(verify_message, str)
            and bool(verify_message.strip())
        )
        if key == "control":
            status = result.get("http_status") if isinstance(result, Mapping) else None
            if kind == "http_response" and type(status) is int:
                acceptances[key] = "PASS" if 200 <= status < 300 else "FAIL"
            elif kind == "tls_certificate_verification_error":
                acceptances[key] = "FAIL"
            else:
                acceptances[key] = "UNJUDGED"
        else:
            if trusted_chain_rejection:
                acceptances[key] = "PASS"
            elif kind == "http_response":
                acceptances[key] = "FAIL"
            else:
                acceptances[key] = "UNJUDGED"
    return acceptances


def aggregate_acceptance(
    acceptances: Mapping[str, str],
    forbidden_matches: Mapping[str, object],
    raw_cafile_status: str,
    inventory_status: str,
) -> str:
    """TLS受入とCA store完全性を合わせてfail-closedに判定する。"""
    if any(value == "FAIL" for value in acceptances.values()):
        return "FAIL"
    if (forbidden_matches.get("distrusted_anchor_der_sha256_present")
            or forbidden_matches.get("impacted_intermediate_spki_sha256_present")):
        return "FAIL"
    absent_spki = forbidden_matches.get("impacted_intermediate_spki_sha256_absent")
    if (forbidden_matches.get("impacted_intermediate_spki_expected_count") != 8
            or not isinstance(absent_spki, list)
            or any(not isinstance(item, str) for item in absent_spki)
            or set(absent_spki) != IMPACTED_INTERMEDIATE_SPKI_SHA256):
        return "UNJUDGED"
    if set(acceptances) != set(HOSTS) or any(value != "PASS" for value in acceptances.values()):
        return "UNJUDGED"
    if raw_cafile_status != "PASS" or inventory_status != "PASS":
        return "UNJUDGED"
    return "PASS"


def peer_certificate(response: object) -> dict[str, str] | None:
    try:
        sock = response.fp.raw._sock
        der = sock.getpeercert(binary_form=True)
    except (AttributeError, OSError):
        return None
    pubkey_der, subject, issuer = certificate_public_key(der, -1)
    return {"subject": subject, "issuer": issuer,
            "der_sha256": hashlib.sha256(der).hexdigest(),
            "spki_sha256": hashlib.sha256(pubkey_der).hexdigest()}


def measure(url: str, opener: urllib.request.OpenerDirector) -> dict[str, object]:
    req = urllib.request.Request(url, headers={"User-Agent": "KCU-Issue13-TLS-Audit/1.0"})
    try:
        with opener.open(req, timeout=TIMEOUT_SECONDS) as response:
            return {"classification": "http_response", "http_status": response.status,
                    "http_reason": response.reason, "peer_certificate": peer_certificate(response)}
    except Exception as error:
        classification, detail, verify_code, verify_message = classify_error(error)
        return {"classification": classification, "detail": detail,
                "verify_code": verify_code, "verify_message": verify_message,
                "http_status": getattr(error, "code", None),
                "peer_certificate": peer_certificate(error) if isinstance(error, urllib.error.HTTPError) else None}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--image-id", required=True)
    args = parser.parse_args()

    ctx = ssl.create_default_context()
    verify = ssl.get_default_verify_paths()
    inventory_status = "PASS"
    inventory_error = None
    try:
        inventory = cert_inventory(ctx)
        inventory_sha256 = canonical_inventory_sha256(inventory)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        inventory = []
        inventory_sha256 = None
        inventory_status = "UNJUDGED"
        inventory_error = f"{type(error).__name__}: {error}"
    raw_cafile = native_cafile_sha256(verify.cafile)
    forbidden_matches = forbidden_inventory_matches(inventory)

    opener = urllib.request.build_opener(
        urllib.request.ProxyHandler({}), NoRedirect(), urllib.request.HTTPSHandler(context=ctx)
    )
    hosts = {key: measure(url, opener) for key, url in HOSTS.items()}
    acceptances = host_acceptances(hosts)
    for key, result in hosts.items():
        result["acceptance"] = acceptances[key]
    aggregate = aggregate_acceptance(acceptances, forbidden_matches, raw_cafile["status"], inventory_status)
    payload = {
        "image_id": args.image_id,
        "runtime": {"python": __import__("platform").python_version(), "openssl": ssl.OPENSSL_VERSION},
        "tls_verification": "enabled; ssl.create_default_context()",
        "proxy": "disabled; direct requests only",
        "redirects": "disabled; only five configured hosts contacted",
        "timeout_seconds_per_host": TIMEOUT_SECONDS,
        "verify_paths": {key: getattr(verify, key) for key in ("cafile", "capath", "openssl_cafile_env", "openssl_cafile", "openssl_capath_env", "openssl_capath")},
        "native_cafile": raw_cafile,
        "trust_anchor_count": len(inventory),
        "trust_anchor_inventory_status": inventory_status,
        "trust_anchor_inventory_sha256": inventory_sha256,
        "trust_anchor_inventory_error": inventory_error,
        "trust_anchors": inventory,
        "impacted_intermediates_source": IMPACTED_INTERMEDIATES_SOURCE,
        "impacted_intermediates_source_snapshot_sha256": IMPACTED_INTERMEDIATES_SOURCE_SNAPSHOT_SHA256,
        "impacted_intermediates_certificate_count": 16,
        "impacted_intermediates_unique_spki_count": 8,
        "impacted_intermediate_spki_sha256": sorted(IMPACTED_INTERMEDIATE_SPKI_SHA256),
        "forbidden_trust_anchor_matches": forbidden_matches,
        "hosts": hosts,
        "aggregate_acceptance": aggregate,
    }
    args.output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    summary = {"image_id": args.image_id, "trust_anchor_count": len(inventory),
               "native_cafile": raw_cafile, "trust_anchor_inventory_sha256": inventory_sha256,
               "forbidden_trust_anchor_matches": forbidden_matches, "hosts": hosts,
               "aggregate_acceptance": aggregate}
    print(json.dumps(summary, sort_keys=True))
    return 0 if aggregate == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
