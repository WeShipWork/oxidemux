## ADDED Requirements

### Requirement: Routing supplies alias context for reasoning normalization

The `oxmux` routing policy and alias primitives SHALL supply enough typed alias context for reasoning normalization to derive supported alias-style reasoning conventions before provider execution while route selection remains responsible only for selecting provider/account targets and reporting routing outcomes. Typed alias reasoning metadata SHALL be attached to alias definitions as structured metadata rather than inferred by parsing suffix, bracket, or free-form model-name conventions.

#### Scenario: Alias context is available before execution

- **WHEN** route selection resolves a requested model alias to a canonical model identifier
- **THEN** reasoning normalization can inspect the requested alias, resolved model identifier, and configured alias metadata without reparsing app-shell state or provider-specific payloads

#### Scenario: Alias metadata is typed rather than parsed from model names

- **WHEN** a routing policy contains an alias that should imply reasoning behavior in this change
- **THEN** the reasoning behavior is represented by typed alias metadata attached to the alias definition rather than by parsing suffixes, brackets, or free-form text from the requested model name

#### Scenario: Route selection does not perform provider rewrites

- **WHEN** a request includes explicit or alias-derived reasoning intent
- **THEN** routing selection preserves requested/resolved model and selected target metadata without generating provider-specific payload rewrites, beta headers, or SDK requests

### Requirement: Reasoning compatibility does not bypass routing failures

Reasoning and thinking compatibility handling in `oxmux` SHALL NOT cause route selection to silently bypass missing routes, exhausted candidates, degraded-only candidates, invalid policy, or explicit-target failures. Reasoning compatibility evaluation SHALL consume the already-selected routing target and SHALL NOT select, prefer, skip, or reroute candidates based on reasoning support in this change.

#### Scenario: Routing failure remains routing failure

- **WHEN** a request includes reasoning intent but routing cannot select a provider/account target
- **THEN** `oxmux` returns the structured routing failure rather than converting it into a reasoning compatibility outcome

#### Scenario: Selected target is used for reasoning compatibility

- **WHEN** route selection succeeds for a request with reasoning intent
- **THEN** reasoning compatibility evaluation uses the selected provider/account/model target metadata and does not select a different route solely because reasoning support differs unless a future routing-policy change explicitly scopes that behavior

#### Scenario: Reasoning support does not trigger fallback selection

- **WHEN** route selection chooses a provider/account/model target that cannot honor requested reasoning intent
- **AND** a lower-priority fallback candidate could honor the requested reasoning intent
- **THEN** this change evaluates reasoning compatibility against the selected target and returns supported, ignored, degraded, unsupported, or unknown outcome data according to handling policy
- **AND** `oxmux` does not select a different route solely because of reasoning support
