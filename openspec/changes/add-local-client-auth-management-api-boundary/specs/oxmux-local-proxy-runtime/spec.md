## MODIFIED Requirements

### Requirement: Runtime dispatches minimal proxy route
The `oxmux` local runtime SHALL preserve the stable health endpoint while also classifying loopback requests by route category and dispatching a bounded loopback `POST /v1/chat/completions` request to the minimal proxy engine path when configured with deterministic core proxy inputs and valid local client authorization when that route is protected.

#### Scenario: Runtime classifies local route categories explicitly
- **WHEN** the runtime receives a loopback request
- **THEN** it classifies `GET /health` as health, `POST /v1/chat/completions` as inference, `/v0/management/*` as management/status/control, and any other method/path as unsupported before applying route behavior

#### Scenario: Health endpoint remains stable
- **WHEN** a client sends `GET /health` to a running local runtime after local client authorization boundaries are added
- **THEN** the runtime still returns the stable health response defined for local health smoke testing without requiring provider, OAuth, quota, credential, GPUI, app-shell, or local client authorization state

#### Scenario: Authorized chat-completion route is dispatched on loopback runtime
- **WHEN** a client sends a valid minimal `POST /v1/chat/completions` request with valid configured local inference authorization to a running loopback runtime configured for mock-backed proxy execution
- **THEN** the runtime dispatches the request to the `oxmux` minimal proxy engine and returns the serialized engine response over the local HTTP connection

#### Scenario: Unauthorized chat-completion route is rejected before proxy execution
- **WHEN** a client sends `POST /v1/chat/completions` without valid local inference authorization and the route is configured as protected
- **THEN** the runtime returns a deterministic unauthorized response without invoking routing or provider execution and without exposing expected credential values

#### Scenario: Management route authorization is distinct from inference authorization
- **WHEN** a client sends a `/v0/management/*` request with only inference authorization
- **THEN** the runtime rejects the request with a deterministic unauthorized response rather than treating it as an inference request or a health check

#### Scenario: Authorized management boundary returns deterministic placeholder response
- **WHEN** a client sends a `/v0/management/*` request with valid configured management/status/control authorization before a concrete management operation is defined
- **THEN** the runtime returns a deterministic protected-boundary response that proves authorization and classification succeeded without invoking inference routing, provider execution, OAuth, platform credential storage, or a remote management panel

#### Scenario: Unauthorized management boundary is rejected
- **WHEN** a client sends a `/v0/management/*` request without valid configured management/status/control authorization
- **THEN** the runtime returns a deterministic unauthorized response without exposing expected credential values and without invoking inference routing or provider execution

#### Scenario: Runtime rejects unsupported local requests deterministically
- **WHEN** a client sends a local request whose method or path is neither `GET /health`, `POST /v1/chat/completions`, nor `/v0/management/*`
- **THEN** the runtime returns a deterministic unsupported-path response without reporting health success, authorization success, management success, or proxy execution success

### Requirement: Runtime request parsing is bounded and local-only
The `oxmux` local runtime SHALL parse only the bounded local HTTP request data needed for the health endpoint, local client authorization, route classification, minimal chat-completion smoke route, and `/v0/management/*` route boundaries, and SHALL reject malformed or oversized requests with deterministic failures instead of panicking or reading unbounded input.

#### Scenario: Malformed local proxy request is rejected
- **WHEN** a loopback client sends malformed request-line, header, body, local authorization, or content data for a local runtime route
- **THEN** the runtime returns a deterministic invalid-request response and keeps the listener usable for later valid requests

#### Scenario: Runtime parses local authorization without retaining unrelated headers
- **WHEN** a loopback client sends a local request with authorization and other headers
- **THEN** the runtime retains only the bounded header data needed for content length and local client authorization decisions and does not expose raw authorization values through status, debug, display, or provider execution surfaces

#### Scenario: Runtime remains loopback-only
- **WHEN** runtime configuration requests a non-loopback listen address after local client authorization boundaries are added
- **THEN** `oxmux` still returns a structured local runtime configuration error instead of binding a public network interface

#### Scenario: Runtime avoids desktop and provider-network dependencies
- **WHEN** maintainers inspect or run local runtime tests for local client authorization and route boundaries
- **THEN** the runtime remains independent from GPUI, tray/background lifecycle, updater, packaging, OAuth UI, token refresh, raw credential storage, provider SDKs, real provider accounts, and outbound provider network calls

#### Scenario: Management boundary remains local and side-effect-free
- **WHEN** maintainers inspect or exercise `/v0/management/*` behavior in this change
- **THEN** the runtime keeps the boundary loopback-only, non-HTML, non-provider-credential-bearing, side-effect-free, and independent from remote management panels until a later OpenSpec change defines concrete management operations

### Requirement: Runtime exposes stable health endpoint
The system SHALL expose a stable health endpoint suitable for smoke testing the minimal local runtime, and adding local client authorization boundaries SHALL NOT change the health response contract or make health checks depend on provider, routing, OAuth, quota, credential, local client authorization, GPUI, or app-shell state.

#### Scenario: Health request succeeds
- **WHEN** a client sends `GET /health` to a running local runtime after local client authorization boundaries are added
- **THEN** the runtime returns the same successful HTTP response with stable content indicating the runtime is healthy

#### Scenario: Unknown path does not masquerade as health
- **WHEN** a client requests a path other than `GET /health`, `POST /v1/chat/completions`, or `/v0/management/*`
- **THEN** the runtime returns a deterministic non-health response that does not report a healthy smoke-test result, authorization success, management success, or proxy execution success
