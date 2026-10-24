## ADDED Requirements

### Requirement: Conversation context follows each ACP transport

The host SHALL preserve conversation continuity when changing ACP providers by supplying the selected transport with messages it has not completed processing. Each transport SHALL maintain its own confirmed history position. Enqueuing a prompt or cancellation alone SHALL NOT confirm that position.

#### Scenario: A conversation switches to another provider and back

- **WHEN** a completed conversation on provider A continues on provider B and later returns to A
- **THEN** B receives the preceding conversation and A receives only the messages after A's confirmed position, while reselecting the same provider does not replay confirmed messages

#### Scenario: A prompt fails or cancellation is still pending

- **WHEN** a prompt has been enqueued but fails, or its cancellation has not reached a prompt terminal response
- **THEN** the confirmed history position remains unchanged, failed staging is discarded without losing conversation data, and the next accepted submission recomputes the unconfirmed context

#### Scenario: A prompt reaches its terminal response

- **WHEN** the outstanding prompt reports Finished, including the terminal response after cancellation
- **THEN** only that transport confirms the staged message position and clears its pending context, without consuming another provider's context

#### Scenario: The host starts or restores a conversation

- **WHEN** NewChat or SelectHistory creates a session boundary, or a replacement snapshot is shorter than a stored history position
- **THEN** the history position starts at zero and the selected conversation's existing messages remain available as context; session boundaries discard the previous transports

#### Scenario: Historical text and file attachments continue on another provider

- **WHEN** an existing message contains saved Text or FileResource attachments, including a message with no body text
- **THEN** the selected provider receives the text label/content and file URI/MIME/saved text together with the message role
- **AND** ACP history keeps every dynamic value escaped within the existing data boundary, and KatanAgent history retains the saved attachment data in the corresponding ChatTurn
- **AND** no file URI is reopened, fetched, or inferred during history projection

#### Scenario: Attachment history is retried or already confirmed

- **WHEN** a provider is reselected, an unconfirmed submission fails, or the host restores a conversation boundary
- **THEN** saved attachments follow the same per-message confirmed cursor as the message body, without replaying confirmed attachments or losing unconfirmed attachments

#### Scenario: A generated or edited document remains available to later turns

- **WHEN** a saved message has structured outputs associated by source_message_id and the conversation continues on KatanAgent or another ACP provider
- **THEN** the corresponding history message includes the saved output kind, status, and payload as escaped data, including generated file content and edited updated content
- **AND** candidate status remains distinct from an applied filesystem change, no path is reopened, output order is preserved, and outputs without a matching message are not assigned to another message
- **AND** outputs follow the same confirmed message cursor and retry/session boundaries as the associated message, without replaying confirmed output data

### Requirement: Permission requests show the proposed operation
The host SHALL preserve the operation information provided in an ACP permission request and show it in the common KUC confirmation surface before the user chooses an option.

#### Scenario: An agent asks to approve a file or command action
- **WHEN** the permission request provides a title, textual operation details, or target paths
- **THEN** the confirmation surface shows the provided action information without executing or interpreting it as HTML
- **AND** the existing request ID, offered option validation, cancellation, and modal input boundary remain intact

#### Scenario: The agent requests several permissions concurrently
- **WHEN** another permission request arrives while a request is awaiting a decision
- **THEN** the host preserves the current request and queues later requests in arrival order
- **AND** a successfully sent decision advances to the next request while a send failure preserves the current request
- **AND** completion, failure, or session cancellation clears the displayed request and queued requests

### Requirement: Model configuration uses the confirmed response
The host SHALL project and persist the configuration returned by a successful model configuration response.

#### Scenario: KatanAgent is selected again after a confirmed model selection

- **WHEN** the user reselects KatanAgent or returns to it after another provider, and its executor retains a confirmed model, including a prompt still executing or completing cancellation
- **THEN** the UI and persisted history retain that executor model, without substituting an unconfirmed restored UI value or inferring a selection from catalog flags
- **AND** a fresh or cleared executor remains unselected, while history restoration still waits for its existing model confirmation process

#### Scenario: The agent does not send a separate configuration notification
- **WHEN** the model configuration response contains updated config options and current values
- **THEN** the host uses the confirmed current model from that response
- **AND** history restoration does not revert to the previous model

#### Scenario: The agent changes the advertised configuration IDs
- **WHEN** a configuration notification or successful configuration response replaces the advertised options
- **THEN** the transport accepts configuration IDs from that same updated option set for the matching session
- **AND** removed or unknown IDs produce a recoverable configuration rejection instead of failing the provider
- **AND** a different session's options do not change the current session's accepted IDs

#### Scenario: Another configuration is selected before confirmation
- **WHEN** a configuration request is awaiting its response and another selection arrives
- **THEN** the transport processes selections in arrival order and publishes each confirmed result before starting the next request
- **AND** permission decisions and cancellation remain available while a configuration response is pending
- **AND** a full bounded queue rejects the extra selection recoverably and a closed worker reports a command failure

#### Scenario: A configuration result arrives while cancellation is pending
- **WHEN** a model configuration confirmation or rejection arrives after Stop and before the outstanding prompt completes
- **THEN** the host projects the confirmed current model or reports the configuration rejection
- **AND** it keeps cancellation pending and preserves the draft and conversation until the prompt terminates
- **AND** residual text, thinking, activity, and permission updates remain suppressed

#### Scenario: A later queued selection is rejected
- **WHEN** a confirmed model update is followed by a recoverable configuration rejection in the same poll
- **THEN** the host persists the confirmed model before reporting the rejection
- **AND** history restoration keeps the confirmed model and a persistence failure is reported explicitly

### Requirement: Cancellation waits for the outstanding prompt
The host SHALL keep an ACP turn active until the outstanding prompt returns a terminal response, even after the cancellation command is queued.

#### Scenario: The user submits another draft immediately after Stop
- **WHEN** Stop has queued cancellation and the outstanding prompt has not completed
- **THEN** the host rejects another submission before changing its draft or conversation
- **AND** residual text, thinking, activity, and permission updates do not alter the cancelled turn
- **AND** prompt completion or terminal failure releases the cancelling state
- **AND** repeated Stop does not complete the prompt locally

#### Scenario: A permission request arrives after Stop
- **WHEN** cancellation has drained existing permission requests and another request arrives for the outstanding prompt
- **THEN** the transport returns Cancelled without exposing a new permission dialog
- **AND** the prompt can terminate and a subsequently accepted prompt can request normal permission again

#### Scenario: Prompt and cancellation commands are processed back to back
- **WHEN** a prompt command is immediately followed by Stop before the prompt background task is polled
- **THEN** the client publishes the prompt request before the cancellation notification
- **AND** cancellation does not wait for the prompt response
- **AND** the host keeps cancellation pending until the outstanding prompt terminates
