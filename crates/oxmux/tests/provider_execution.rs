use std::net::{IpAddr, Ipv4Addr};

use oxmux::{
    AuthMethodCategory, AuthState, CanonicalProtocolRequest, CanonicalProtocolResponse,
    ConfigurationSnapshot, CoreError, CoreHealthState, DegradedReason, ManagementSnapshot,
    MockProviderAccount, MockProviderHarness, MockProviderOutcome, ProtocolFamily,
    ProtocolMetadata, ProtocolPayload, ProtocolPayloadBody, ProtocolResponseStatus,
    ProviderExecutionFailure, ProviderExecutionOutcome, ProviderExecutionRequest, ProviderExecutor,
    ProxyLifecycleState, QuotaState, QuotaSummary, ResponseMode, RoutingDefault, StreamContent,
    StreamEvent, StreamTerminalState, StreamingResponse, UsageSummary, core_identity,
};

#[test]
fn mock_provider_returns_success_without_translation_or_network() -> Result<(), CoreError> {
    let response = canonical_response(ProtocolMetadata::provider_specific(
        "mock-openai",
        "responses-json",
    )?)?;
    let executor = MockProviderHarness::new(
        "mock-openai",
        "Mock OpenAI",
        ProtocolFamily::ProviderSpecific,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Success(response.clone()),
    )?
    .with_account(MockProviderAccount::new("acct-primary", "Primary account"));

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-openai",
        Some("acct-primary".to_string()),
        canonical_request(ProtocolMetadata::provider_specific(
            "mock-openai",
            "responses-json",
        )?)?,
    )?)?;

    assert_eq!(result.outcome.complete_response(), Some(&response));
    assert_eq!(result.metadata.provider.provider_id, "mock-openai");
    assert!(!result.metadata.provider.capabilities[0].supports_streaming);
    assert!(result.metadata.provider.capabilities[0].routing_eligible);
    assert_eq!(
        result
            .metadata
            .account
            .expect("account metadata")
            .account_id,
        "acct-primary"
    );
    assert_eq!(response.protocol.family(), ProtocolFamily::ProviderSpecific);

    Ok(())
}

#[test]
fn mock_provider_returns_degraded_metadata_with_canonical_response() -> Result<(), CoreError> {
    let degraded_reason = DegradedReason {
        component: "provider:mock-claude".to_string(),
        message: "provider reports stale quota data".to_string(),
    };
    let executor = MockProviderHarness::new(
        "mock-claude",
        "Mock Claude",
        ProtocolFamily::Claude,
        AuthMethodCategory::OAuth,
        MockProviderOutcome::Degraded {
            response: canonical_response(ProtocolMetadata::claude())?,
            reasons: vec![degraded_reason.clone()],
        },
    )?
    .with_account(MockProviderAccount::new(
        "acct-degraded",
        "Degraded account",
    ));

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-claude",
        Some("acct-degraded".to_string()),
        canonical_request(ProtocolMetadata::claude())?,
    )?)?;

    assert!(matches!(
        result.outcome,
        ProviderExecutionOutcome::Degraded { ref reasons, .. }
            if reasons == std::slice::from_ref(&degraded_reason)
    ));
    assert_eq!(
        result.metadata.provider.degraded_reasons,
        std::slice::from_ref(&degraded_reason)
    );
    assert_eq!(
        result
            .metadata
            .account
            .expect("account metadata")
            .degraded_reasons,
        [degraded_reason]
    );

    Ok(())
}

#[test]
fn mock_provider_reflects_quota_limited_state_in_account_summary() -> Result<(), CoreError> {
    let quota_state = QuotaState::Limited {
        remaining: 0,
        limit: 100,
    };
    let executor = MockProviderHarness::new(
        "mock-gemini",
        "Mock Gemini",
        ProtocolFamily::Gemini,
        AuthMethodCategory::ExternalReference,
        MockProviderOutcome::QuotaLimited {
            response: canonical_response(ProtocolMetadata::gemini())?,
            quota_state: quota_state.clone(),
        },
    )?
    .with_account(
        MockProviderAccount::new("acct-quota", "Quota limited account")
            .with_quota_state(QuotaState::Unlimited),
    );

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-gemini",
        Some("acct-quota".to_string()),
        canonical_request(ProtocolMetadata::gemini())?,
    )?)?;

    assert!(matches!(
        result.outcome,
        ProviderExecutionOutcome::QuotaLimited { ref quota_state, .. }
            if quota_state == &QuotaState::Limited { remaining: 0, limit: 100 }
    ));
    assert_eq!(
        result
            .metadata
            .account
            .expect("account metadata")
            .quota_state,
        quota_state
    );

    Ok(())
}

#[test]
fn streaming_capable_mock_reports_capability_without_stream_transport() -> Result<(), CoreError> {
    let executor = MockProviderHarness::new(
        "mock-codex",
        "Mock Codex",
        ProtocolFamily::Codex,
        AuthMethodCategory::None,
        MockProviderOutcome::complete_streaming_capable(canonical_response(
            ProtocolMetadata::codex(),
        )?),
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-codex",
        None,
        canonical_request(ProtocolMetadata::codex())?,
    )?)?;

    assert!(matches!(
        result.outcome,
        ProviderExecutionOutcome::Success(_)
    ));
    assert_eq!(
        result
            .outcome
            .complete_response()
            .expect("complete response"),
        &canonical_response(ProtocolMetadata::codex())?
    );
    assert!(result.metadata.provider.capabilities[0].supports_streaming);
    assert!(result.metadata.account.is_none());

    Ok(())
}

#[test]
fn mock_provider_returns_deterministic_streaming_response_events() -> Result<(), CoreError> {
    let streaming_response = StreamingResponse::new(vec![
        StreamEvent::Content(StreamContent::new(
            ProtocolMetadata::open_ai(),
            ProtocolPayload::opaque("application/json", br#"{"delta":"first"}"#.to_vec()),
        )?),
        StreamEvent::Content(StreamContent::new(
            ProtocolMetadata::open_ai(),
            ProtocolPayload::opaque("application/json", br#"{"delta":"second"}"#.to_vec()),
        )?),
        StreamEvent::Terminal(StreamTerminalState::completed()),
    ])?;
    let executor = MockProviderHarness::new(
        "mock-streaming",
        "Mock Streaming",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::streaming(streaming_response.clone()),
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-streaming",
        None,
        canonical_request(ProtocolMetadata::open_ai())?,
    )?)?;

    assert!(matches!(
        result.outcome.response_mode(),
        ResponseMode::Streaming(response) if response == &streaming_response
    ));
    assert_eq!(
        result
            .outcome
            .response_mode()
            .streaming_response()
            .expect("streaming response")
            .events(),
        streaming_response.events()
    );
    assert!(result.metadata.provider.capabilities[0].supports_streaming);

    Ok(())
}

#[test]
fn mock_provider_delivers_cancelled_stream_as_response_data() -> Result<(), CoreError> {
    let streaming_response = StreamingResponse::new(vec![StreamEvent::Terminal(
        StreamTerminalState::cancelled(oxmux::CancellationReason::Timeout),
    )])?;
    let executor = MockProviderHarness::new(
        "mock-cancelled-stream",
        "Mock Cancelled Stream",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::streaming(streaming_response),
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-cancelled-stream",
        None,
        canonical_request(ProtocolMetadata::open_ai())?,
    )?)?;

    assert!(matches!(
        result
            .outcome
            .response_mode()
            .streaming_response()
            .and_then(|response| response.terminal()),
        Some(StreamTerminalState::Cancelled {
            reason: oxmux::CancellationReason::Timeout,
        })
    ));

    Ok(())
}

#[test]
fn mock_provider_delivers_errored_stream_as_response_data() -> Result<(), CoreError> {
    let failure = oxmux::StreamFailure::new("upstream_error", "upstream ended stream")?;
    let streaming_response = StreamingResponse::new(vec![StreamEvent::Terminal(
        StreamTerminalState::errored(failure.clone()),
    )])?;
    let executor = MockProviderHarness::new(
        "mock-errored-stream",
        "Mock Errored Stream",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::streaming(streaming_response),
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-errored-stream",
        None,
        canonical_request(ProtocolMetadata::open_ai())?,
    )?)?;

    assert!(matches!(
        result.outcome.response_mode().streaming_response().and_then(|response| response.terminal()),
        Some(StreamTerminalState::Errored { failure: returned_failure }) if returned_failure == &failure
    ));

    Ok(())
}

#[test]
fn complete_mock_outcome_can_report_streaming_capability() -> Result<(), CoreError> {
    let response = canonical_response(ProtocolMetadata::claude())?;
    let executor = MockProviderHarness::new(
        "mock-complete-streaming-capable",
        "Mock Complete Streaming Capable",
        ProtocolFamily::Claude,
        AuthMethodCategory::OAuth,
        MockProviderOutcome::complete_streaming_capable(response.clone()),
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-complete-streaming-capable",
        None,
        canonical_request(ProtocolMetadata::claude())?,
    )?)?;

    assert_eq!(result.outcome.complete_response(), Some(&response));
    assert!(result.metadata.provider.capabilities[0].supports_streaming);

    Ok(())
}

#[test]
fn complete_degraded_mock_outcome_can_report_streaming_capability() -> Result<(), CoreError> {
    let degraded_reason = DegradedReason {
        component: "provider:mock-degraded-streaming-capable".to_string(),
        message: "provider degraded while streaming-capable".to_string(),
    };
    let response = canonical_response(ProtocolMetadata::claude())?;
    let executor = MockProviderHarness::new(
        "mock-degraded-streaming-capable",
        "Mock Degraded Streaming Capable",
        ProtocolFamily::Claude,
        AuthMethodCategory::OAuth,
        MockProviderOutcome::DegradedWithMode {
            response_mode: ResponseMode::complete(response.clone()),
            supports_streaming: true,
            reasons: vec![degraded_reason.clone()],
        },
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-degraded-streaming-capable",
        None,
        canonical_request(ProtocolMetadata::claude())?,
    )?)?;

    assert_eq!(result.outcome.complete_response(), Some(&response));
    assert!(matches!(
        result.outcome,
        ProviderExecutionOutcome::Degraded { ref reasons, .. }
            if reasons == std::slice::from_ref(&degraded_reason)
    ));
    assert!(result.metadata.provider.capabilities[0].supports_streaming);

    Ok(())
}

#[test]
fn complete_quota_limited_mock_outcome_can_report_streaming_capability() -> Result<(), CoreError> {
    let response = canonical_response(ProtocolMetadata::gemini())?;
    let quota_state = QuotaState::Limited {
        remaining: 3,
        limit: 100,
    };
    let executor = MockProviderHarness::new(
        "mock-quota-streaming-capable",
        "Mock Quota Streaming Capable",
        ProtocolFamily::Gemini,
        AuthMethodCategory::ExternalReference,
        MockProviderOutcome::QuotaLimitedWithMode {
            response_mode: ResponseMode::complete(response.clone()),
            supports_streaming: true,
            quota_state: quota_state.clone(),
        },
    )?;

    let result = executor.execute(ProviderExecutionRequest::new(
        "mock-quota-streaming-capable",
        None,
        canonical_request(ProtocolMetadata::gemini())?,
    )?)?;

    assert_eq!(result.outcome.complete_response(), Some(&response));
    assert!(matches!(
        result.outcome,
        ProviderExecutionOutcome::QuotaLimited { ref quota_state, .. }
            if quota_state == &QuotaState::Limited { remaining: 3, limit: 100 }
    ));
    assert!(result.metadata.provider.capabilities[0].supports_streaming);

    Ok(())
}

#[test]
fn failed_mock_provider_returns_structured_provider_execution_error() -> Result<(), CoreError> {
    let executor = MockProviderHarness::new(
        "mock-failure",
        "Mock Failure",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Failed(ProviderExecutionFailure::failed_outcome(
            "provider_unavailable",
            "provider unavailable in mock outcome",
        )),
    )?
    .with_account(
        MockProviderAccount::new("acct-failed", "Failed account").with_auth_state(
            AuthState::Failed {
                reason: "mock failure".to_string(),
            },
        ),
    );

    let error = executor.execute(ProviderExecutionRequest::new(
        "mock-failure",
        Some("acct-failed".to_string()),
        canonical_request(ProtocolMetadata::open_ai())?,
    )?);

    assert!(matches!(
        error,
        Err(CoreError::ProviderExecution {
            provider_id,
            account_id: Some(account_id),
            failure: ProviderExecutionFailure::FailedOutcome { code, .. },
        }) if provider_id == "mock-failure"
            && account_id == "acct-failed"
            && code == "provider_unavailable"
    ));
    assert_eq!(
        executor.provider_summary().degraded_reasons,
        [DegradedReason {
            component: "provider_execution".to_string(),
            message: "provider unavailable in mock outcome".to_string(),
        }]
    );

    Ok(())
}

#[test]
fn provider_execution_contracts_are_usable_through_public_facade()
-> Result<(), Box<dyn std::error::Error>> {
    let executor = oxmux::MockProviderHarness::new(
        "facade-provider",
        "Facade Provider",
        oxmux::ProtocolFamily::OpenAi,
        oxmux::AuthMethodCategory::ApiKey,
        oxmux::MockProviderOutcome::Success(oxmux::CanonicalProtocolResponse::new(
            oxmux::ProtocolMetadata::open_ai(),
            oxmux::ProtocolResponseStatus::success(),
            oxmux::ProtocolPayload::empty(),
        )?),
    )?;
    let request = oxmux::ProviderExecutionRequest::new(
        "facade-provider",
        None,
        oxmux::CanonicalProtocolRequest::new(
            oxmux::ProtocolMetadata::open_ai(),
            "facade-model",
            oxmux::ProtocolPayload::empty(),
        )?,
    )?;

    let result = oxmux::ProviderExecutor::execute(&executor, request)?;

    assert!(matches!(
        result.outcome,
        oxmux::ProviderExecutionOutcome::Success(_)
    ));

    Ok(())
}

#[test]
fn management_snapshot_can_include_mock_provider_health() -> Result<(), CoreError> {
    let degraded_reason = DegradedReason {
        component: "provider:mock-health".to_string(),
        message: "mock provider is degraded".to_string(),
    };
    let executor = MockProviderHarness::new(
        "mock-health",
        "Mock Health",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Degraded {
            response: canonical_response(ProtocolMetadata::open_ai())?,
            reasons: vec![degraded_reason.clone()],
        },
    )?;
    let provider = executor.provider_summary();
    let snapshot = ManagementSnapshot {
        identity: core_identity(),
        lifecycle: ProxyLifecycleState::Stopped,
        health: CoreHealthState::Degraded {
            reasons: vec![degraded_reason.clone()],
        },
        configuration: ConfigurationSnapshot {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8787,
            auto_start: false,
            logging_enabled: true,
            usage_collection_enabled: false,
            routing_default: RoutingDefault::named("manual"),
            provider_references: vec!["mock-health".to_string()],
        },
        file_configuration: None,
        last_configuration_load_failure: None,
        providers: vec![provider],
        usage: UsageSummary::zero(),
        quota: QuotaSummary::unknown(),
        warnings: vec!["mock provider is degraded".to_string()],
        errors: Vec::new(),
    };

    assert_eq!(snapshot.providers[0].provider_id, "mock-health");
    assert!(matches!(snapshot.health, CoreHealthState::Degraded { .. }));
    assert_eq!(snapshot.providers[0].degraded_reasons, [degraded_reason]);

    Ok(())
}

#[test]
fn provider_execution_rejects_routing_and_credential_substitution() -> Result<(), CoreError> {
    let executor = MockProviderHarness::new(
        "explicit-provider",
        "Explicit Provider",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Success(canonical_response(ProtocolMetadata::open_ai())?),
    )?
    .with_account(MockProviderAccount::new(
        "explicit-account",
        "Explicit account",
    ))
    .with_routing_eligible(false);

    let error = executor.execute(ProviderExecutionRequest::new(
        "explicit-provider",
        Some("other-account".to_string()),
        canonical_request(ProtocolMetadata::open_ai())?,
    )?);

    assert!(matches!(
        error,
        Err(CoreError::ProviderExecution {
            failure: ProviderExecutionFailure::InvalidSelection { .. },
            ..
        })
    ));
    assert!(!executor.provider_summary().capabilities[0].routing_eligible);

    Ok(())
}

fn canonical_request(protocol: ProtocolMetadata) -> Result<CanonicalProtocolRequest, CoreError> {
    CanonicalProtocolRequest::new(
        protocol,
        "mock-model",
        ProtocolPayload::opaque("application/json", br#"{"input":"hello"}"#.to_vec()),
    )
}

fn canonical_response(protocol: ProtocolMetadata) -> Result<CanonicalProtocolResponse, CoreError> {
    CanonicalProtocolResponse::new(
        protocol,
        ProtocolResponseStatus::success(),
        ProtocolPayload {
            content_type: Some("application/json".to_string()),
            body: ProtocolPayloadBody::Opaque(br#"{"output":"ok"}"#.to_vec()),
        },
    )
}
