## ADDED Requirements

### Requirement: Permission metadata uses paired ACP wire events

The metadata tracker SHALL count a permission request only when it observes an ACP 3.6.0 / schema 1.13.0 JSON-RPC `session/request_permission` request from the agent. It SHALL resolve that request only from the opposite-direction JSON-RPC response with the same request ID. The tracker SHALL assign an arrival ordinal and record observation times and the unanswered-request count without exposing raw request IDs.

#### Scenario: A permission request is matched to its response

- **WHEN** the tracker observes an actual `session/request_permission` request and later observes its opposite-direction JSON-RPC result or error response with the same ID
- **THEN** the request and response are associated through their ordinal, their observation times are recorded, and the unanswered-request count is updated
- **AND** the request ID, session ID, option ID, and request or response contents are not emitted or persisted

#### Scenario: A tool call does not create a permission request

- **WHEN** a `tool_call` update is observed without an actual `session/request_permission` request
- **THEN** the permission-request count remains unchanged and the tracker does not label the event as an approval wait

#### Scenario: Unrelated or malformed messages do not resolve a request

- **WHEN** the tracker observes an unknown ID, wrong-direction response, duplicate response, malformed JSON-RPC message, normal prompt terminal, cancel notification, or unrelated RPC event
- **THEN** no different permission request is resolved and no raw message data is emitted

#### Scenario: Responses arrive out of order for concurrent requests

- **WHEN** multiple permission requests are outstanding and their responses arrive in a different order
- **THEN** each response resolves only the request with the matching ID while public records retain request-arrival ordinals and the unanswered-request count reflects outstanding requests

#### Scenario: The metadata component is imported or invoked

- **WHEN** the metadata module is imported or its parser is invoked on captured events
- **THEN** it does not start a CLI, provider, child process, or network request
- **AND** output contains no raw request/session/option IDs, prompt or response text, tool arguments, paths, headers, or authentication values

### Requirement: Permission observation does not change runtime policy

The metadata tracker SHALL remain observational. It SHALL NOT change prompt timeout, marker evaluation, permission decisions, commercial code, dependencies, UI rendering, or existing quality gates.

#### Scenario: Historical evidence is incomplete

- **WHEN** a historical run contains no captured permission request/response wire events
- **THEN** the tracker and its documentation do not infer that the run was waiting for approval or rewrite its recorded outcome

#### Scenario: Existing permission and cancellation policy remains active

- **WHEN** metadata is collected for a permission or cancellation event
- **THEN** the existing runtime permission decision and cancellation behavior remain unchanged
