## ADDED Requirements

### Requirement: File configuration represents streaming robustness policy
The `oxmux` core SHALL load and validate file-backed streaming robustness configuration for keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior without depending on `oxidemux`, GPUI, provider SDKs, OAuth UI, credential storage, remote configuration services, databases, filesystem watchers, or live upstream streams.

The canonical TOML table SHALL be `[streaming]`. Supported fields SHALL be `keepalive-interval-ms`, `bootstrap-retry-count`, `timeout-ms`, and `cancellation`. Duration fields use integer milliseconds and SHALL accept values from `1` through `300000` inclusive. Explicit `0` for duration fields SHALL be invalid; omission disables the behavior. `bootstrap-retry-count` is the number of additional attempts after the initial attempt and SHALL accept values from `0` through `10` inclusive. Negative numbers, floats, strings for numeric fields, and integer overflow SHALL be invalid. Supported cancellation values SHALL be `"disabled"`, `"client-disconnect"`, and `"timeout"`. `cancellation = "timeout"` SHALL require a configured `timeout-ms`; `timeout-ms` with `cancellation = "disabled"` SHALL record timeout policy/metadata without automatically converting timeout into cancellation. `cancellation = "client-disconnect"` SHALL configure deterministic core policy and mock outcomes only; live request-context disconnect propagation is deferred to a later transport change.

Omitted `[streaming]` configuration SHALL produce deterministic disabled defaults: no keepalive interval, `bootstrap-retry-count = 0`, no timeout, and `cancellation = "disabled"`. Unknown streaming fields or nested tables SHALL be rejected under the existing strict configuration model.

#### Scenario: Streaming policy can be configured from TOML
- **WHEN** a Rust consumer loads a syntactically valid local TOML configuration containing supported streaming robustness fields
- **THEN** `oxmux` returns typed configuration data that represents keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior for core runtime consumers

#### Scenario: Streaming policy accepts canonical field names
- **WHEN** a Rust consumer loads TOML containing `[streaming]` with `keepalive-interval-ms`, `bootstrap-retry-count`, `timeout-ms`, and `cancellation`
- **THEN** `oxmux` maps those fields into the typed streaming robustness policy without requiring app-shell configuration types

#### Scenario: Streaming policy defaults are explicit
- **WHEN** a TOML configuration omits streaming robustness settings
- **THEN** `oxmux` produces deterministic default streaming policy values rather than relying on transport-specific implicit defaults

#### Scenario: Streaming policy fields can be partially omitted
- **WHEN** a TOML configuration includes `[streaming]` with only some supported fields
- **THEN** `oxmux` applies disabled defaults for omitted fields and validates supplied fields normally

#### Scenario: Invalid streaming configuration is structured
- **WHEN** loaded configuration contains an unsupported keepalive interval, bootstrap retry count, timeout value, or cancellation behavior
- **THEN** validation fails with structured configuration error data identifying the streaming field path and invalid value category
- **AND** field paths use the canonical names, such as `streaming.keepalive-interval-ms`, `streaming.bootstrap-retry-count`, `streaming.timeout-ms`, and `streaming.cancellation`

#### Scenario: Streaming numeric ranges are enforced
- **WHEN** loaded configuration contains `0`, a negative number, a float, a string, or an overflow-sized integer for `streaming.keepalive-interval-ms` or `streaming.timeout-ms`
- **THEN** validation fails with structured configuration error data for the affected duration field
- **AND** omission remains the only way to disable a duration behavior

#### Scenario: Streaming retry count range is enforced
- **WHEN** loaded configuration contains a negative number, a number greater than `10`, a float, a string, or an overflow-sized integer for `streaming.bootstrap-retry-count`
- **THEN** validation fails with structured configuration error data for `streaming.bootstrap-retry-count`
- **AND** `streaming.bootstrap-retry-count = 0` remains valid and means no additional attempts

#### Scenario: Timeout cancellation requires timeout policy
- **WHEN** loaded configuration sets `streaming.cancellation = "timeout"` without `streaming.timeout-ms`
- **THEN** validation fails with structured configuration error data identifying the missing timeout policy relationship

#### Scenario: Timeout policy can be metadata-only
- **WHEN** loaded configuration sets `streaming.timeout-ms` and leaves cancellation omitted or sets `streaming.cancellation = "disabled"`
- **THEN** validation succeeds and represents timeout metadata policy without automatic timeout-driven cancellation

#### Scenario: Invalid streaming configuration stays in configuration error taxonomy
- **WHEN** loaded configuration contains an unsupported streaming policy value
- **THEN** validation returns `CoreError::Configuration` data rather than `CoreError::Streaming`

#### Scenario: Bootstrap retry count distinguishes retries from attempts
- **WHEN** a TOML configuration sets `streaming.bootstrap-retry-count = 2`
- **THEN** the resulting policy allows at most two additional bootstrap attempts after the initial streaming attempt

#### Scenario: Unknown streaming configuration fields remain rejected
- **WHEN** a TOML configuration contains an unknown streaming robustness table or field
- **THEN** `oxmux` returns a structured unknown-field configuration error rather than silently ignoring the value

#### Scenario: Layered configuration preserves streaming validation
- **WHEN** file-backed configuration layers or fixtures merge streaming policy values
- **THEN** the merged result validates the same canonical field paths, defaults, unknown-field rejection, and structured error categories as a single TOML file
