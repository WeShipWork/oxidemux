## 1. Reasoning Core Types

- [ ] 1.1 Add a focused `reasoning` module with provider-neutral intent, source, mode, effort, budget, handling policy, capability, compatibility outcome, and diagnostic types.
- [ ] 1.2 Add deterministic validation for typed Rust reasoning metadata, reasoning budgets, the `1..=200000` provider-neutral budget range, effort/budget mutual exclusivity, effort/mode combinations, absent intent, and conflicting explicit controls.
- [ ] 1.3 Add structured reasoning validation and unsupported-capability failures to `CoreError` without requiring display-string parsing.
- [ ] 1.4 Export stable reasoning primitives through the public `oxmux` facade with rustdoc coverage.

## 2. Alias and Capability Semantics

- [ ] 2.1 Add narrow deterministic in-memory typed alias-derived reasoning metadata support while preserving requested alias and resolved model identity separately; defer persisted TOML config and suffix/bracket/free-form alias parsing.
- [ ] 2.2 Add provider/account/model reasoning capability metadata that can represent support, limits, degraded support, unsupported state, and unknown state while keeping ignored as a compatibility outcome rather than a capability state.
- [ ] 2.3 Ensure reasoning compatibility evaluation uses selected provider/account/model metadata without changing route selection behavior, does not reroute to reasoning-capable fallbacks, and treats unknown capability as strict failure for explicit intent or ignored metadata for permissive alias/default intent.
- [ ] 2.4 Extend model registry candidate metadata to expose reasoning/thinking capability information without implying live provider discovery.

## 3. Protocol and Provider Boundary Integration

- [ ] 3.1 Extend canonical protocol request metadata so normalized reasoning intent can travel separately from opaque payload bodies without parsing OpenAI/Gemini/Claude/Codex JSON fields in this change.
- [ ] 3.2 Ensure deferred protocol translation preserves reasoning metadata and does not claim provider-specific rewrites occurred.
- [ ] 3.3 Extend provider execution request and metadata primitives to carry normalized reasoning intent plus supported/ignored/degraded/unsupported/unknown compatibility outcomes.
- [ ] 3.4 Keep mock provider execution deterministic and free of provider-specific beta headers, SDK requests, OAuth, credential storage, and network calls.

## 4. Tests and Verification

- [ ] 4.1 Add unit tests for explicit typed Rust reasoning metadata, absent intent, valid effort, valid budget, invalid budget, effort/budget mutual exclusivity, and conflicting controls.
- [ ] 4.2 Add tests for typed alias-derived reasoning metadata, ordinary aliases with no reasoning intent, invalid alias metadata, explicit-over-alias precedence diagnostics, and no suffix/bracket/free-form alias parsing.
- [ ] 4.3 Add tests for supported, strict unsupported, permissive ignored, degraded, unknown reasoning capability outcomes, model/account/provider capability precedence, and no reroute to a reasoning-capable fallback.
- [ ] 4.4 Add integration-style tests proving protocol/provider execution metadata propagation remains networkless, provider-SDK-free, and independent from opaque payload JSON parsing.
- [ ] 4.5 Run targeted `cargo test -p oxmux reasoning` or equivalent focused tests.
- [ ] 4.6 Run `mise run ci` or the documented relevant subset and record results.
