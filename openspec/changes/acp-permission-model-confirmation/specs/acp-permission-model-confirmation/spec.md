## ADDED Requirements

### Requirement: Permission requests show the proposed operation
The host SHALL preserve the operation information provided in an ACP permission request and show it in the common KUC confirmation surface before the user chooses an option.

#### Scenario: An agent asks to approve a file or command action
- **WHEN** the permission request provides a title, textual operation details, or target paths
- **THEN** the confirmation surface shows the provided action information without executing or interpreting it as HTML
- **AND** the existing request ID, offered option validation, cancellation, and modal input boundary remain intact

### Requirement: Model configuration uses the confirmed response
The host SHALL project and persist the configuration returned by a successful model configuration response.

#### Scenario: The agent does not send a separate configuration notification
- **WHEN** the model configuration response contains updated config options and current values
- **THEN** the host uses the confirmed current model from that response
- **AND** history restoration does not revert to the previous model
