"""ACP観測行から機密を保持せず、順序・待ち数だけを記録する純粋tracker。"""

from __future__ import annotations

import json
import threading
import time
from collections.abc import Mapping


_DIRECTIONS = {"client_to_agent", "agent_to_client"}
_TOOL_STATUSES = {"pending", "in_progress", "completed", "failed"}
_STOP_REASONS = {"end_turn", "max_tokens", "max_turn_requests", "refusal", "cancelled"}


class MetadataTracker:
    """JSON-RPC metadataだけを保持し、IDや本文は出力しない。"""

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._sequence = 0
        self._ordinal = 0
        self._origin_ns: int | None = None
        self._outstanding: dict[tuple[str, str, str | int], int] = {}
        self._events: list[dict[str, object]] = []

    def ingest(
        self,
        direction: str,
        raw_line: str | bytes,
        *,
        monotonic_ns: int | None = None,
    ) -> dict[str, object]:
        """1行を処理して、その行に対応する安全なmetadata eventを返す。"""
        if isinstance(raw_line, bytes):
            raw_bytes = raw_line
            try:
                decoded = raw_line.decode("utf-8")
            except UnicodeDecodeError:
                decoded = None
        else:
            decoded = raw_line
            raw_bytes = raw_line.encode("utf-8", errors="replace")

        with self._lock:
            now_ns = time.monotonic_ns() if monotonic_ns is None else monotonic_ns
            if self._origin_ns is None:
                self._origin_ns = now_ns
            elapsed_ns = max(0, now_ns - self._origin_ns)
            message: object | None = None
            if decoded is not None:
                try:
                    message = json.loads(
                        decoded,
                        object_pairs_hook=self._unique_object,
                        parse_constant=self._reject_constant,
                    )
                except (ValueError, RecursionError):
                    message = None

            safe_direction = direction if direction in _DIRECTIONS else "other"
            event: dict[str, object] = {
                "direction": safe_direction,
                "elapsed_ns": elapsed_ns,
                "byte_count": len(raw_bytes),
                "kind": "other",
                "status": "other",
            }
            message_object = self._mapping(message)
            if message_object is None:
                event["kind"] = "invalid_json"
                self._append(event)
                return dict(event)
            if message_object.get("jsonrpc") != "2.0":
                event["kind"] = "invalid_rpc"
                self._append(event)
                return dict(event)

            method = message_object.get("method")
            request_id = message_object.get("id", _MISSING)
            if method == "session/request_permission" and request_id is not _MISSING:
                id_key = self._id_key(request_id)
                if (
                    safe_direction != "agent_to_client"
                    or id_key is None
                    or "result" in message_object
                    or "error" in message_object
                    or not self._permission_request_valid(message_object.get("params"))
                ):
                    event["kind"] = "permission_request_invalid"
                else:
                    key = (safe_direction, id_key[0], id_key[1])
                    if key in self._outstanding:
                        event["kind"] = "permission_request_duplicate"
                    else:
                        self._ordinal += 1
                        self._outstanding[key] = self._ordinal
                        event["kind"] = "permission_request"
                        event["status"] = "received"
                        event["request_ordinal"] = self._ordinal
            elif request_id is not _MISSING and "method" not in message_object:
                result = self._mapping(message_object.get("result"))
                error = self._mapping(message_object.get("error"))
                has_valid_result = "result" in message_object and "error" not in message_object
                has_valid_error = (
                    "error" in message_object
                    and "result" not in message_object
                    and error is not None
                    and isinstance(error.get("code"), int)
                    and not isinstance(error.get("code"), bool)
                    and isinstance(error.get("message"), str)
                )
                if has_valid_result and result is not None and "stopReason" in result:
                    event["kind"] = "prompt_result"
                    event["stop_reason"] = self._allowlisted(result.get("stopReason"), _STOP_REASONS)
                elif has_valid_error or (
                    has_valid_result and self._response_status(message_object) != "other"
                ):
                    id_key = self._id_key(request_id)
                    if safe_direction in _DIRECTIONS and id_key is not None:
                        expected_request_direction = self._opposite(safe_direction)
                        key = (expected_request_direction, id_key[0], id_key[1])
                        ordinal = self._outstanding.pop(key, None)
                    else:
                        ordinal = None
                    if ordinal is not None:
                        event["kind"] = "permission_response"
                        event["request_ordinal"] = ordinal
                        event["status"] = self._response_status(message_object)
            elif method == "session/prompt":
                event["kind"] = "prompt_request"
            elif method == "session/cancel":
                event["kind"] = "cancel_notification"
            elif method == "session/update":
                event.update(self._classify_update(message_object.get("params")))

            self._append(event)
            return dict(event)

    def snapshot(self) -> dict[str, object]:
        """保存可能なイベントと待ち数だけを返す。"""
        with self._lock:
            return {
                "events": [dict(event) for event in self._events],
                "outstanding_count": len(self._outstanding),
                "request_count": self._ordinal,
                "sequence_count": self._sequence,
            }

    def _append(self, event: dict[str, object]) -> None:
        self._sequence += 1
        event["sequence"] = self._sequence
        event["outstanding_count"] = len(self._outstanding)
        self._events.append(event)

    @staticmethod
    def _unique_object(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Duplicate JSON key")
            result[key] = value
        return result

    @staticmethod
    def _reject_constant(value: str) -> None:
        raise ValueError("Nonstandard JSON constant")

    @staticmethod
    def _mapping(value: object) -> Mapping[str, object] | None:
        return value if isinstance(value, Mapping) else None

    @classmethod
    def _permission_request_valid(cls, params: object) -> bool:
        request = cls._mapping(params)
        if request is None or not isinstance(request.get("sessionId"), str):
            return False
        tool_call = cls._mapping(request.get("toolCall"))
        options = request.get("options")
        if tool_call is None or not isinstance(tool_call.get("toolCallId"), str):
            return False
        if not isinstance(options, list):
            return False
        for value in options:
            option = cls._mapping(value)
            if (
                option is None
                or not isinstance(option.get("optionId"), str)
                or not isinstance(option.get("name"), str)
                or cls._allowlisted(
                    option.get("kind"), {"allow_once", "allow_always", "reject_once", "reject_always"}
                ) == "other"
            ):
                return False
        return True

    @staticmethod
    def _id_key(value: object) -> tuple[str, str | int] | None:
        if isinstance(value, str):
            return ("str", value)
        if isinstance(value, int) and not isinstance(value, bool):
            return ("int", value)
        return None

    @staticmethod
    def _opposite(direction: str) -> str:
        return "agent_to_client" if direction == "client_to_agent" else "client_to_agent"

    @staticmethod
    def _allowlisted(value: object, allowed: set[str]) -> str:
        return value if isinstance(value, str) and value in allowed else "other"

    @classmethod
    def _response_status(cls, message: Mapping[str, object]) -> str:
        if "error" in message:
            return "rpcerror"
        result = cls._mapping(message.get("result"))
        if result is None:
            return "other"
        outcome = cls._mapping(result.get("outcome"))
        if outcome is None:
            return "other"
        if outcome.get("outcome") == "selected" and isinstance(outcome.get("optionId"), str):
            return "selected"
        return "cancelled" if outcome.get("outcome") == "cancelled" else "other"

    @classmethod
    def _classify_update(cls, params: object) -> dict[str, object]:
        notification = cls._mapping(params)
        update = cls._mapping(notification.get("update")) if notification is not None else None
        if update is None:
            return {"kind": "other", "status": "other"}
        update_kind = update.get("sessionUpdate")
        if update_kind == "agent_message_chunk":
            return {"kind": "agent_message", "status": "other"}
        if update_kind == "agent_thought_chunk":
            return {"kind": "agent_thought", "status": "other"}
        if update_kind == "tool_call":
            return {"kind": "tool_call", "status": cls._allowlisted(update.get("status"), _TOOL_STATUSES)}
        if update_kind == "tool_call_update":
            return {"kind": "tool_call", "status": cls._allowlisted(update.get("status"), _TOOL_STATUSES)}
        return {"kind": "other", "status": "other"}


_MISSING = object()
