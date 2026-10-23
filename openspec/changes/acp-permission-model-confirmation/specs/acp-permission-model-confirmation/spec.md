## ADDED Requirements

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
