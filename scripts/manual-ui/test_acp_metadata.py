"""純粋metadata trackerの境界テスト。"""

from __future__ import annotations

import ast
import json
from pathlib import Path
import threading
import unittest

from acp_metadata import MetadataTracker


def rpc(value: object) -> str:
    return json.dumps(value, separators=(",", ":"))


def permission_request(request_id: object, session_id: str = "secret-session") -> str:
    return rpc({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "session/request_permission",
        "params": {
            "sessionId": session_id,
            "toolCall": {"toolCallId": "secret-tool-call", "status": "pending"},
            "options": [{"optionId": "secret-option", "name": "secret-prompt", "kind": "allow_once"}],
            "_meta": {
                "headers": {"Authorization": "SECRET_AUTH_HEADER"},
                "auth": "SECRET_AUTH_VALUE", "path": "SECRET_FILE_PATH",
            },
        },
    })


class MetadataTrackerTest(unittest.TestCase):
    def test_duplicate_json_keys_do_not_create_or_resolve_requests(self) -> None:
        tracker = MetadataTracker()
        request = permission_request(1).replace('"id":1', '"id":2,"id":1', 1)
        invalid_request = tracker.ingest("agent_to_client", request)
        self.assertEqual(invalid_request["kind"], "invalid_json")
        self.assertEqual(invalid_request["outstanding_count"], 0)
        tracker.ingest("agent_to_client", permission_request(1))
        invalid_response = tracker.ingest("client_to_agent", (
            '{"jsonrpc":"2.0","id":2,"id":1,'
            '"result":{"outcome":{"outcome":"cancelled"}}}'
        ))
        self.assertEqual(invalid_response["kind"], "invalid_json")
        self.assertEqual(invalid_response["outstanding_count"], 1)

    def test_nonstandard_json_constants_do_not_resolve_request(self) -> None:
        for constant in ("NaN", "Infinity", "-Infinity"):
            with self.subTest(constant_length=len(constant)):
                tracker = MetadataTracker()
                tracker.ingest("agent_to_client", permission_request(1))
                event = tracker.ingest("client_to_agent", (
                    '{"jsonrpc":"2.0","id":1,"result":{"outcome":'
                    '{"outcome":"cancelled"}},"_meta":' + constant + '}'
                ))
                self.assertEqual(event["outstanding_count"], 1)

    def test_request_with_response_fields_is_not_counted(self) -> None:
        for field in ("result", "error"):
            with self.subTest(field=field):
                tracker = MetadataTracker()
                request = json.loads(permission_request(1))
                request[field] = {}
                event = tracker.ingest("agent_to_client", rpc(request))
                self.assertEqual(event["kind"], "permission_request_invalid")
                self.assertEqual(event["outstanding_count"], 0)
                self.assertEqual(tracker.snapshot()["request_count"], 0)

    def test_invalid_rpc_error_does_not_consume_pending_request(self) -> None:
        tracker = MetadataTracker()
        tracker.ingest("agent_to_client", permission_request(1))
        event = tracker.ingest("client_to_agent", rpc({"jsonrpc": "2.0", "id": 1, "error": {}}))
        self.assertEqual(event["kind"], "other")
        self.assertEqual(event["outstanding_count"], 1)

    def test_malformed_outcomes_do_not_consume_pending_request(self) -> None:
        for outcome in (None, "selected", {"outcome": "selected"}, {"outcome": "unknown"}):
            with self.subTest(outcome=outcome):
                tracker = MetadataTracker()
                tracker.ingest("agent_to_client", permission_request(1))
                event = tracker.ingest("client_to_agent", rpc({
                    "jsonrpc": "2.0", "id": 1, "result": {"outcome": outcome},
                }))
                self.assertEqual(event["status"], "other")
                self.assertEqual(event["kind"], "other")
                self.assertEqual(event["outstanding_count"], 1)
                valid = tracker.ingest("client_to_agent", rpc({
                    "jsonrpc": "2.0", "id": 1,
                    "result": {"outcome": {"outcome": "cancelled"}},
                }))
                self.assertEqual(valid["request_ordinal"], 1)
                self.assertEqual(valid["outstanding_count"], 0)

    def test_duplicate_messages_and_typed_ids_do_not_resolve_another_request(self) -> None:
        tracker = MetadataTracker()
        tracker.ingest("agent_to_client", permission_request(1))
        tracker.ingest("agent_to_client", permission_request("1"))
        duplicate = tracker.ingest("agent_to_client", permission_request(1))
        self.assertEqual(duplicate["kind"], "permission_request_duplicate")
        self.assertEqual(duplicate["outstanding_count"], 2)
        self.assertEqual(tracker.snapshot()["request_count"], 2)
        reply = {"jsonrpc": "2.0", "id": 1, "result": {"outcome": {"outcome": "cancelled"}}}
        first = tracker.ingest("client_to_agent", rpc(reply))
        self.assertEqual(first["request_ordinal"], 1)
        duplicate_reply = tracker.ingest("client_to_agent", rpc(reply))
        self.assertEqual(duplicate_reply["outstanding_count"], 1)
        reply["id"] = "1"
        second = tracker.ingest("client_to_agent", rpc(reply))
        self.assertEqual(second["request_ordinal"], 2)
        self.assertEqual(second["outstanding_count"], 0)

    def test_concurrent_permission_pairs_keep_local_ordinals_and_close_independently(self) -> None:
        tracker = MetadataTracker()
        barrier = threading.Barrier(8)

        def worker(index: int) -> None:
            barrier.wait()
            tracker.ingest("agent_to_client", permission_request(index))
            tracker.ingest("client_to_agent", rpc({
                "jsonrpc": "2.0", "id": index,
                "result": {"outcome": {"outcome": "cancelled"}},
            }))

        threads = [threading.Thread(target=worker, args=(i,)) for i in range(8)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        snapshot = tracker.snapshot()
        self.assertEqual(snapshot["request_count"], 8)
        self.assertEqual(snapshot["outstanding_count"], 0)
        self.assertEqual(snapshot["sequence_count"], 16)
        events = snapshot["events"]
        self.assertEqual([event["sequence"] for event in events], list(range(1, 17)))
        elapsed = [event["elapsed_ns"] for event in events]
        self.assertEqual(elapsed, sorted(elapsed))

    def test_invalid_permission_direction_or_shape_is_not_pending(self) -> None:
        tracker = MetadataTracker()
        event = tracker.ingest("client_to_agent", permission_request("SECRET_ID"))
        self.assertEqual(event["kind"], "permission_request_invalid")
        malformed = json.loads(permission_request("SECRET_ID"))
        malformed["params"] = {}
        event = tracker.ingest("agent_to_client", rpc(malformed))
        self.assertEqual(event["kind"], "permission_request_invalid")
        self.assertEqual(tracker.snapshot()["request_count"], 0)
        self.assertEqual(tracker.snapshot()["outstanding_count"], 0)
        self.assertNotIn("SECRET_ID", json.dumps(tracker.snapshot()))

    def test_non_rpc_message_cannot_change_pending_permissions(self) -> None:
        tracker = MetadataTracker()
        invalid_request = json.loads(permission_request("private-id"))
        invalid_request["jsonrpc"] = "1.0"
        event = tracker.ingest("agent_to_client", rpc(invalid_request))
        self.assertEqual(event["kind"], "invalid_rpc")
        self.assertEqual(event["outstanding_count"], 0)
        tracker.ingest("agent_to_client", permission_request("private-id"))
        event = tracker.ingest("client_to_agent", rpc({
            "id": "private-id", "result": {"outcome": {"outcome": "cancelled"}},
        }))
        self.assertEqual(event["kind"], "invalid_rpc")
        self.assertEqual(event["outstanding_count"], 1)

    def test_prompt_and_cancel_have_explicit_phase_markers(self) -> None:
        tracker = MetadataTracker()
        prompt = tracker.ingest("client_to_agent", rpc({
            "jsonrpc": "2.0", "id": "SECRET_ID", "method": "session/prompt",
            "params": {"sessionId": "SECRET_SESSION", "prompt": [
                {"type": "text", "text": "SECRET_PROMPT"},
            ]},
        }))
        cancel = tracker.ingest("client_to_agent", rpc({
            "jsonrpc": "2.0", "method": "session/cancel",
            "params": {"sessionId": "SECRET_SESSION"},
        }))
        self.assertEqual(prompt["kind"], "prompt_request")
        self.assertEqual(cancel["kind"], "cancel_notification")
        for sentinel in ("SECRET_ID", "SECRET_SESSION", "SECRET_PROMPT"):
            self.assertNotIn(sentinel, json.dumps(tracker.snapshot()))

    def test_module_has_no_process_network_or_cli_entrypoint(self) -> None:
        module = ast.parse(Path(__file__).with_name("acp_metadata.py").read_text())
        allowed = {"__future__", "json", "threading", "time", "collections.abc"}
        for node in ast.walk(module):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    self.assertIn(alias.name, allowed)
            elif isinstance(node, ast.ImportFrom):
                self.assertIn(node.module, allowed)
            elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                self.assertNotIn(node.func.id, {"open", "exec", "eval", "__import__", "input"})
            elif isinstance(node, ast.Name):
                self.assertNotEqual(node.id, "__name__")

    def test_acp_36_nested_session_updates_are_classified(self) -> None:
        tracker = MetadataTracker()
        agent_message = tracker.ingest(
            "agent_to_client",
            rpc({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {"sessionId": "secret-session", "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": "SECRET_MESSAGE"},
                }},
            }),
            monotonic_ns=1,
        )
        agent_thought = tracker.ingest(
            "agent_to_client",
            rpc({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {"sessionId": "secret-session", "update": {
                    "sessionUpdate": "agent_thought_chunk",
                    "content": {"type": "text", "text": "SECRET_THOUGHT"},
                }},
            }),
            monotonic_ns=2,
        )
        tool_call = tracker.ingest(
            "agent_to_client",
            rpc({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {"sessionId": "secret-session", "update": {
                    "sessionUpdate": "tool_call",
                    "toolCallId": "secret-tool-id",
                    "status": "in_progress",
                    "title": "SECRET_TITLE",
                }},
            }),
            monotonic_ns=3,
        )
        tool_update = tracker.ingest(
            "agent_to_client",
            rpc({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {"sessionId": "secret-session", "update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "secret-tool-id",
                    "status": "completed",
                    "rawOutput": "SECRET_OUTPUT",
                }},
            }),
            monotonic_ns=4,
        )
        self.assertEqual(agent_message["kind"], "agent_message")
        self.assertEqual(agent_thought["kind"], "agent_thought")
        self.assertEqual(tool_call["status"], "in_progress")
        self.assertEqual(tool_update["status"], "completed")
        serialized = json.dumps(tracker.snapshot())
        for sentinel in ("SECRET_MESSAGE", "SECRET_THOUGHT", "secret-session", "secret-tool-id", "SECRET_TITLE", "SECRET_OUTPUT"):
            self.assertNotIn(sentinel, serialized)

    def test_prompt_and_other_rpc_responses_are_not_permission_responses(self) -> None:
        tracker = MetadataTracker()
        pending = tracker.ingest("agent_to_client", permission_request("permission-id"), monotonic_ns=1)
        initialize = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "id": "init-id", "result": {"protocolVersion": 1}}),
            monotonic_ns=2,
        )
        new_session = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "id": "new-session-id", "result": {"sessionId": "secret-session"}}),
            monotonic_ns=3,
        )
        prompt = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "id": "prompt-id", "result": {"stopReason": "end_turn"}}),
            monotonic_ns=4,
        )
        self.assertEqual(pending["kind"], "permission_request")
        self.assertEqual(initialize["kind"], "other")
        self.assertEqual(new_session["kind"], "other")
        self.assertEqual(prompt["kind"], "prompt_result")
        self.assertEqual(prompt["stop_reason"], "end_turn")
        self.assertEqual(tracker.snapshot()["outstanding_count"], 1)

    def test_id_only_invalid_object_does_not_consume_pending_permission(self) -> None:
        tracker = MetadataTracker()
        tracker.ingest("agent_to_client", permission_request("still-pending"), monotonic_ns=1)
        malformed_response = tracker.ingest(
            "client_to_agent", rpc({"jsonrpc": "2.0", "id": "still-pending"}), monotonic_ns=2
        )
        self.assertEqual(malformed_response["kind"], "other")
        self.assertEqual(tracker.snapshot()["outstanding_count"], 1)

    def test_multiple_requests_match_out_of_order_and_track_counts(self) -> None:
        tracker = MetadataTracker()
        first = tracker.ingest("agent_to_client", permission_request("private-1"), monotonic_ns=20)
        second = tracker.ingest("agent_to_client", permission_request("private-2"), monotonic_ns=30)
        self.assertEqual((first["request_ordinal"], second["request_ordinal"]), (1, 2))
        self.assertEqual(second["outstanding_count"], 2)

        response = tracker.ingest(
            "client_to_agent",
            rpc({"jsonrpc": "2.0", "id": "private-2", "result": {"outcome": {"outcome": "selected", "optionId": "private-option"}}}),
            monotonic_ns=40,
        )
        self.assertEqual((response["request_ordinal"], response["status"]), (2, "selected"))
        self.assertEqual(response["outstanding_count"], 1)
        self.assertEqual(tracker.snapshot()["outstanding_count"], 1)
        serialized = json.dumps(tracker.snapshot())
        for sentinel in (
            "private-1", "private-2", "private-option", "secret-session", "secret-prompt", "secret-option",
            "SECRET_AUTH_HEADER", "SECRET_AUTH_VALUE", "SECRET_FILE_PATH",
        ):
            self.assertNotIn(sentinel, serialized)

    def test_cancel_and_rpc_error_are_terminal_categories(self) -> None:
        tracker = MetadataTracker()
        tracker.ingest("agent_to_client", permission_request(1), monotonic_ns=1)
        cancelled = tracker.ingest(
            "client_to_agent",
            rpc({"jsonrpc": "2.0", "id": 1, "result": {"outcome": {"outcome": "cancelled"}}}),
            monotonic_ns=2,
        )
        tracker.ingest("agent_to_client", permission_request(2), monotonic_ns=3)
        errored = tracker.ingest(
            "client_to_agent",
            rpc({"jsonrpc": "2.0", "id": 2, "error": {"code": -32000, "message": "secret-error"}}),
            monotonic_ns=4,
        )
        self.assertEqual(cancelled["status"], "cancelled")
        self.assertEqual(errored["status"], "rpcerror")
        self.assertEqual(tracker.snapshot()["outstanding_count"], 0)

    def test_unknown_id_and_wrong_direction_do_not_close_request(self) -> None:
        tracker = MetadataTracker()
        tracker.ingest("agent_to_client", permission_request("private-id"), monotonic_ns=1)
        unmatched = tracker.ingest(
            "client_to_agent", rpc({"jsonrpc": "2.0", "id": "unknown-id", "result": {"outcome": {"outcome": "cancelled"}}}), monotonic_ns=2
        )
        wrong_way = tracker.ingest(
            "agent_to_client", rpc({"jsonrpc": "2.0", "id": "private-id", "result": {"outcome": {"outcome": "cancelled"}}}), monotonic_ns=3
        )
        self.assertEqual(unmatched["kind"], "other")
        self.assertEqual(wrong_way["kind"], "other")
        self.assertEqual(tracker.snapshot()["outstanding_count"], 1)

    def test_invalid_json_and_unknown_values_are_sanitized(self) -> None:
        tracker = MetadataTracker()
        invalid = tracker.ingest("client_to_agent", b"{not-json private-marker", monotonic_ns=10)
        unknown = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "method": "secret-method", "params": {"stopReason": "secret-reason"}}),
            monotonic_ns=11,
        )
        prompt = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "id": "secret-prompt-id", "result": {"stopReason": "secret-stop"}}),
            monotonic_ns=12,
        )
        self.assertEqual(invalid["kind"], "invalid_json")
        self.assertEqual(unknown["kind"], "other")
        self.assertEqual(prompt["stop_reason"], "other")
        serialized = json.dumps(tracker.snapshot(), ensure_ascii=False)
        for sentinel in ("private-marker", "secret-method", "secret-reason", "secret-stop"):
            self.assertNotIn(sentinel, serialized)

    def test_tool_status_and_message_emit_metadata_only(self) -> None:
        tracker = MetadataTracker()
        message = rpc({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {"sessionId": "secret-session", "update": {
                "sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": "SECRET_BODY"},
            }},
        })
        tool = rpc({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {"sessionId": "secret-session", "update": {
                "sessionUpdate": "tool_call", "toolCallId": "secret-tool-id", "status": "completed",
                "rawInput": {"path": "SECRET_PATH"},
            }},
        })
        unknown_status = rpc({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {"sessionId": "secret-session", "update": {
                "sessionUpdate": "tool_call_update", "status": "SECRET_STATUS",
            }},
        })
        message_event = tracker.ingest("agent_to_client", message, monotonic_ns=5)
        tool_event = tracker.ingest("agent_to_client", tool, monotonic_ns=6)
        unknown_event = tracker.ingest("agent_to_client", unknown_status, monotonic_ns=7)
        self.assertEqual(message_event["kind"], "agent_message")
        self.assertEqual(message_event["byte_count"], len(message.encode()))
        self.assertEqual(tool_event["status"], "completed")
        self.assertEqual(unknown_event["status"], "other")
        serialized = json.dumps(tracker.snapshot())
        for sentinel in ("SECRET_BODY", "SECRET_PATH", "SECRET_STATUS"):
            self.assertNotIn(sentinel, serialized)

    def test_tool_call_alone_is_not_a_permission_wait(self) -> None:
        tracker = MetadataTracker()
        event = tracker.ingest(
            "agent_to_client",
            rpc({"jsonrpc": "2.0", "method": "session/update", "params": {
                "sessionId": "secret-session", "update": {
                    "sessionUpdate": "tool_call", "status": "in_progress",
                },
            }}),
            monotonic_ns=8,
        )
        self.assertEqual(event["kind"], "tool_call")
        self.assertEqual(event["outstanding_count"], 0)

    def test_sequence_and_monotonic_elapsed_are_local(self) -> None:
        tracker = MetadataTracker()
        one = tracker.ingest("client_to_agent", "{}", monotonic_ns=100)
        two = tracker.ingest("client_to_agent", "{}", monotonic_ns=155)
        self.assertEqual((one["sequence"], two["sequence"]), (1, 2))
        self.assertEqual((one["elapsed_ns"], two["elapsed_ns"]), (0, 55))

    def test_concurrent_updates_are_serialized(self) -> None:
        tracker = MetadataTracker()
        barrier = threading.Barrier(8)

        def worker(index: int) -> None:
            barrier.wait()
            tracker.ingest("agent_to_client", rpc({"jsonrpc": "2.0", "method": "other"}), monotonic_ns=100 + index)

        threads = [threading.Thread(target=worker, args=(index,)) for index in range(8)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        snapshot = tracker.snapshot()
        self.assertEqual(snapshot["sequence_count"], 8)
        self.assertEqual(sorted(event["sequence"] for event in snapshot["events"]), list(range(1, 9)))
