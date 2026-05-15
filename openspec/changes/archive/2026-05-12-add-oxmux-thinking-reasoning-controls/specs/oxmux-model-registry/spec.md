## ADDED Requirements

### Requirement: Model registry exposes reasoning capability metadata

The `oxmux` model registry SHALL be able to expose provider-neutral reasoning and thinking capability metadata for listed model candidates so future listing consumers, compatibility checks, and app-shell model pickers can inspect support without duplicating alias parsing or provider-specific rewrite semantics. Registry metadata SHALL project typed alias reasoning metadata and model-candidate capability metadata supplied by core inputs; it SHALL NOT parse suffix, bracket, or free-form model-name reasoning conventions.

#### Scenario: Listed model candidate reports reasoning support

- **WHEN** a model registry entry is associated with a provider/account/model candidate that declares reasoning or thinking support
- **THEN** the registry exposes typed capability metadata for supported modes, effort levels, budget support, degraded reasons, unsupported reasons, or unknown state without requiring live provider discovery

#### Scenario: Registry preserves typed alias metadata for reasoning controls

- **WHEN** a listed model entry was created from an in-memory model alias that declares typed reasoning metadata
- **THEN** the registry preserves requested alias, resolved model, alias reasoning metadata, and candidate metadata so reasoning controls can consume typed registry data without reparsing flattened model strings in `oxidemux`

#### Scenario: Registry aliases do not parse model-name reasoning syntax

- **WHEN** a listed model identifier contains suffixes, brackets, parentheses, or other free-form text that a future compatibility convention might interpret as reasoning syntax
- **THEN** this change treats the identifier as an ordinary model identifier unless typed alias reasoning metadata is supplied by core inputs

#### Scenario: Registry does not imply live reasoning discovery

- **WHEN** reasoning capability metadata is unavailable or unknown for a configured model candidate
- **THEN** model listing represents that state as typed unknown or unsupported metadata rather than implying that provider credentials, quota, or live model discovery were checked
