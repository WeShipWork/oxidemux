## ADDED Requirements

### Requirement: Routing policy supplies model registry metadata
Routing policy primitives SHALL supply model registry construction with alias, resolved-model, route group, provider/account target, fallback, and candidate-order metadata while preserving routing selection as a separate operation.

#### Scenario: Registry construction consumes aliases without selecting a route
- **WHEN** a routing policy contains a model alias and one or more routes for the resolved model
- **THEN** model registry construction can expose the alias and resolved model metadata without evaluating provider availability or selecting a target

#### Scenario: Registry construction preserves candidate order
- **WHEN** a routing policy contains multiple candidates for the same resolved model
- **THEN** model registry construction preserves deterministic candidate order for listing and fork metadata

#### Scenario: Registry construction does not mutate routing policy
- **WHEN** a Rust consumer builds model registry entries from a routing policy
- **THEN** the routing policy remains reusable for later `RoutingBoundary::select` calls without mutated fallback, availability, or skipped-candidate state

#### Scenario: Registry metadata distinguishes eligibility from selection
- **WHEN** a candidate is routing-ineligible, disabled, degraded, or unavailable according to supplied metadata
- **THEN** the registry exposes that listing state while route selection continues to return routing-specific selection results or structured routing failures
