#!/usr/bin/env python3
"""コンテナTLS検査の純粋な分類・受入判定を検証する。"""
from __future__ import annotations

import importlib.util
import json
import pathlib
import socket
import ssl
import sys
import unittest
import urllib.error

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
PROBE_SPEC = importlib.util.spec_from_file_location("check_container_tls", SCRIPT_DIR / "check-container-tls.py")
WRAPPER_SPEC = importlib.util.spec_from_file_location("check_images", SCRIPT_DIR / "check-images.py")
assert PROBE_SPEC is not None and PROBE_SPEC.loader is not None
assert WRAPPER_SPEC is not None and WRAPPER_SPEC.loader is not None
probe = importlib.util.module_from_spec(PROBE_SPEC)
wrapper = importlib.util.module_from_spec(WRAPPER_SPEC)
sys.modules[PROBE_SPEC.name] = probe
sys.modules[WRAPPER_SPEC.name] = wrapper
PROBE_SPEC.loader.exec_module(probe)
WRAPPER_SPEC.loader.exec_module(wrapper)


def passing_hosts() -> dict[str, dict[str, object]]:
    return {
        "control": {"classification": "http_response", "http_status": 200},
        "g3": {"classification": "tls_certificate_verification_error", "verify_code": 20, "verify_message": "unable to get local issuer certificate"},
        "aaa": {"classification": "tls_certificate_verification_error", "verify_code": 20, "verify_message": "unable to get local issuer certificate"},
        "noct": {"classification": "tls_certificate_verification_error", "verify_code": 20, "verify_message": "unable to get local issuer certificate"},
        "aaa_noct": {"classification": "tls_certificate_verification_error", "verify_code": 20, "verify_message": "unable to get local issuer certificate"},
    }


def verification_error(code: int | None, message: str | None) -> ssl.SSLCertVerificationError:
    error = ssl.SSLCertVerificationError(1, message or "certificate verify failed")
    error.verify_code = code
    error.verify_message = message
    return error


def absent_forbidden_matches() -> dict[str, object]:
    return {
        "distrusted_anchor_der_sha256_present": [],
        "impacted_intermediate_spki_sha256_present": [],
        "impacted_intermediate_spki_sha256_absent": sorted(probe.IMPACTED_INTERMEDIATE_SPKI_SHA256),
        "impacted_intermediate_spki_expected_count": 8,
    }


class ErrorClassificationTests(unittest.TestCase):
    def test_control_requires_successful_two_hundred_http_response(self) -> None:
        for status, expected in ((200, "PASS"), (204, "PASS"), (302, "FAIL"), (404, "FAIL"), (500, "FAIL")):
            with self.subTest(status=status):
                hosts = passing_hosts()
                hosts["control"] = {"classification": "http_response", "http_status": status}
                self.assertEqual(expected, probe.host_acceptances(hosts)["control"])

    def test_control_without_http_status_is_unjudged(self) -> None:
        hosts = passing_hosts()
        hosts["control"] = {"classification": "http_response"}
        self.assertEqual("UNJUDGED", probe.host_acceptances(hosts)["control"])

    def test_any_http_response_from_rejection_hosts_fails(self) -> None:
        for status in (200, 404, 500):
            with self.subTest(status=status):
                hosts = passing_hosts()
                hosts["g3"] = {"classification": "http_response", "http_status": status}
                self.assertEqual("FAIL", probe.host_acceptances(hosts)["g3"])

    def test_dns_failure_is_not_tls_rejection(self) -> None:
        classification, _, _, _ = probe.classify_error(urllib.error.URLError(socket.gaierror("name lookup failed")))
        self.assertEqual(classification, "dns_error")

    def test_connection_refusal_is_not_tls_rejection(self) -> None:
        classification, _, _, _ = probe.classify_error(urllib.error.URLError(ConnectionRefusedError("refused")))
        self.assertEqual(classification, "connection_error")

    def test_timeout_is_not_tls_rejection(self) -> None:
        classification, _, _, _ = probe.classify_error(urllib.error.URLError(socket.timeout("timed out")))
        self.assertEqual(classification, "timeout")

    def test_certificate_verification_error_is_distinguished(self) -> None:
        error = verification_error(20, "unable to get local issuer certificate")
        classification, detail, verify_code, verify_message = probe.classify_error(error)
        self.assertEqual(classification, "tls_certificate_verification_error")
        self.assertIn("local issuer", detail)
        self.assertEqual(20, verify_code)
        self.assertEqual("unable to get local issuer certificate", verify_message)

    def test_nested_url_error_preserves_chain_verification_code_and_message(self) -> None:
        error = verification_error(21, "unable to verify the first certificate")
        classification, _, verify_code, verify_message = probe.classify_error(urllib.error.URLError(error))
        self.assertEqual("tls_certificate_verification_error", classification)
        self.assertEqual(21, verify_code)
        self.assertEqual("unable to verify the first certificate", verify_message)

    def test_only_observed_local_issuer_failure_code_passes(self) -> None:
        hosts = passing_hosts()
        hosts["g3"] = {
            "classification": "tls_certificate_verification_error",
            "verify_code": 20,
            "verify_message": "unable to get local issuer certificate",
        }
        self.assertEqual("PASS", probe.host_acceptances(hosts)["g3"])

    def test_hostname_expiry_purpose_and_missing_codes_remain_unjudged(self) -> None:
        for code in (2, 10, 18, 19, 21, 23, 26, 27, 28, 62, None):
            with self.subTest(code=code):
                hosts = passing_hosts()
                hosts["g3"] = {
                    "classification": "tls_certificate_verification_error",
                    "verify_code": code,
                    "verify_message": "verification failed",
                }
                self.assertEqual("UNJUDGED", probe.host_acceptances(hosts)["g3"])

    def test_missing_verify_message_does_not_pass_chain_rejection(self) -> None:
        hosts = passing_hosts()
        hosts["g3"] = {"classification": "tls_certificate_verification_error", "verify_code": 20}
        self.assertEqual("UNJUDGED", probe.host_acceptances(hosts)["g3"])

    def test_unrelated_certificate_verification_error_cannot_pass_aggregate(self) -> None:
        hosts = passing_hosts()
        hosts["g3"] = {
            "classification": "tls_certificate_verification_error",
            "verify_code": 62,
            "verify_message": "hostname mismatch",
        }
        status = probe.aggregate_acceptance(
            probe.host_acceptances(hosts), absent_forbidden_matches(), "PASS", "PASS",
        )
        self.assertEqual("UNJUDGED", status)

    def test_non_tls_failures_leave_all_host_acceptance_unjudged(self) -> None:
        for kind in ("dns_error", "connection_error", "timeout"):
            with self.subTest(kind=kind):
                hosts = passing_hosts()
                hosts["g3"] = {"classification": kind}
                self.assertEqual(probe.host_acceptances(hosts)["g3"], "UNJUDGED")


class TrustStoreAcceptanceTests(unittest.TestCase):
    def test_all_eight_impacted_spkis_are_explicitly_checked(self) -> None:
        self.assertEqual(len(probe.IMPACTED_INTERMEDIATE_SPKI_SHA256), 8)
        result = probe.forbidden_inventory_matches([])
        self.assertEqual(result["impacted_intermediate_spki_expected_count"], 8)
        self.assertEqual(result["impacted_intermediate_spki_sha256_absent"], sorted(probe.IMPACTED_INTERMEDIATE_SPKI_SHA256))
        self.assertEqual(probe.IMPACTED_INTERMEDIATES_SOURCE,
                         "https://emudhra.we-have-a-problem.com/impacted-intermediates.json")
        self.assertEqual(probe.IMPACTED_INTERMEDIATES_SOURCE_SNAPSHOT_SHA256,
                         "f67b3abe9b4879f4e91874ed041beccd71257f1db41c588c724ccf12aa90c59e")

    def test_canonical_inventory_hash_is_order_independent(self) -> None:
        first = [
            {"der_sha256": "b", "spki_sha256": "2"},
            {"der_sha256": "a", "spki_sha256": "1"},
        ]
        self.assertEqual(probe.canonical_inventory_sha256(first),
                         probe.canonical_inventory_sha256(list(reversed(first))))

    def test_unavailable_native_cafile_hash_is_unjudged_without_a_digest(self) -> None:
        result = probe.native_cafile_sha256(None)
        self.assertEqual(result["status"], "UNJUDGED")
        self.assertIsNone(result["sha256"])

    def test_either_forbidden_anchor_fails_even_when_every_host_passes(self) -> None:
        for fingerprint in sorted(probe.FORBIDDEN_TRUST_ANCHOR_DER_SHA256):
            with self.subTest(fingerprint=fingerprint):
                match = {
                    "distrusted_anchor_der_sha256_present": [fingerprint],
                    "impacted_intermediate_spki_sha256_present": [],
                    "impacted_intermediate_spki_sha256_absent": sorted(probe.IMPACTED_INTERMEDIATE_SPKI_SHA256),
                    "impacted_intermediate_spki_expected_count": 8,
                }
                status = probe.aggregate_acceptance(probe.host_acceptances(passing_hosts()), match, "PASS", "PASS")
                self.assertEqual(status, "FAIL")

    def test_impacted_intermediate_spki_fails_even_when_every_host_passes(self) -> None:
        match = absent_forbidden_matches()
        match["impacted_intermediate_spki_sha256_present"] = [sorted(probe.IMPACTED_INTERMEDIATE_SPKI_SHA256)[0]]
        status = probe.aggregate_acceptance(probe.host_acceptances(passing_hosts()), match, "PASS", "PASS")
        self.assertEqual(status, "FAIL")

    def test_missing_bundle_hash_is_unjudged_even_when_every_host_passes(self) -> None:
        status = probe.aggregate_acceptance(probe.host_acceptances(passing_hosts()), absent_forbidden_matches(), "UNJUDGED", "PASS")
        self.assertEqual(status, "UNJUDGED")

    def test_any_dns_or_connection_failure_keeps_aggregate_unjudged(self) -> None:
        for kind in ("dns_error", "connection_error", "timeout"):
            with self.subTest(kind=kind):
                hosts = passing_hosts()
                hosts["aaa"] = {"classification": kind}
                status = probe.aggregate_acceptance(probe.host_acceptances(hosts), absent_forbidden_matches(), "PASS", "PASS")
                self.assertEqual(status, "UNJUDGED")


class WrapperEvidenceTests(unittest.TestCase):
    def test_git_routing_environment_is_removed_for_source_identity(self) -> None:
        env = wrapper._git_environment({"GIT_DIR": "/untrusted/other.git", "HOME": "/tmp/home"})
        self.assertNotIn("GIT_DIR", env)
        self.assertEqual("/tmp/home", env["HOME"])
        self.assertEqual("0", env["GIT_OPTIONAL_LOCKS"])

    def test_runtime_prefix_uses_absolute_rtk_and_proxy_when_present(self) -> None:
        prefix = wrapper.runtime_prefix("/opt/homebrew/bin/rtk")
        self.assertEqual(prefix, ["/opt/homebrew/bin/rtk", "proxy", "docker"])
        self.assertTrue(pathlib.Path(prefix[0]).is_absolute())

    def test_runtime_prefix_uses_plain_docker_when_rtk_is_unavailable(self) -> None:
        self.assertEqual(wrapper.runtime_prefix(None), ["docker"])
        self.assertEqual(wrapper.runtime_prefix(""), ["docker"])

    def test_absence_requires_successful_exact_id_query_with_empty_output(self) -> None:
        cid = "a" * 64
        self.assertTrue(wrapper.container_is_absent(0, "", cid))
        self.assertFalse(wrapper.container_is_absent(1, "", cid))
        self.assertFalse(wrapper.container_is_absent(0, cid + "\n", cid))
        self.assertFalse(wrapper.container_is_absent(0, "", "short"))

    def test_cleanup_ownership_requires_exact_id_image_and_both_labels(self) -> None:
        cid = "a" * 64
        image_id = "sha256:" + "b" * 64
        owner = "owner-token"
        run = "run-token"
        record = {
            "Id": cid,
            "Image": image_id,
            "State": {"Running": True},
            "Config": {
                "Labels": {
                    wrapper.OWNER_LABEL: owner,
                    wrapper.RUN_LABEL: run,
                },
            },
        }
        matched, _ = wrapper.owned_container_matches([record], cid, image_id, owner, run)
        self.assertTrue(matched)

        for changed_cid, changed_image, labels in (
            ("c" * 64, image_id, record["Config"]["Labels"]),
            (cid, "sha256:" + "d" * 64, record["Config"]["Labels"]),
            (cid, image_id, {wrapper.OWNER_LABEL: owner, wrapper.RUN_LABEL: "other-run"}),
        ):
            with self.subTest(cid=changed_cid, image=changed_image, labels=labels):
                changed = {**record, "Id": changed_cid, "Image": changed_image,
                           "Config": {"Labels": labels}}
                matched, _ = wrapper.owned_container_matches([changed], cid, image_id, owner, run)
                self.assertFalse(matched)

    def test_image_evidence_discards_non_ssl_environment_values(self) -> None:
        image = {
            "Id": "sha256:" + "a" * 64,
            "Os": "linux",
            "Architecture": "arm64",
            "RepoTags": ["candidate:local"],
            "RootFS": {"Type": "layers", "Layers": []},
            "Config": {"Env": ["TOKEN=do-not-save", "SSL_CERT_FILE=/custom/ca.pem"]},
        }
        saved = wrapper.sanitized_image_record(image)
        self.assertEqual(saved["ssl_env_override_names"], ["SSL_CERT_FILE"])
        self.assertNotIn("Config", saved)
        self.assertNotIn("TOKEN", json_text(saved))


def json_text(value: object) -> str:
    return json.dumps(value)


if __name__ == "__main__":
    unittest.main()
