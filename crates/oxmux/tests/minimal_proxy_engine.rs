use std::cell::RefCell;

use oxmux::{
    AuthMethodCategory, CanonicalProtocolResponse, CoreError, FallbackBehavior,
    MINIMAL_PROXY_JSON_CONTENT_TYPE, MinimalProxyEngine, MinimalProxyEngineConfig,
    MinimalProxyErrorCode, MinimalProxyRequest, MockProviderAccount, MockProviderHarness,
    MockProviderOutcome, ModelRoute, ProtocolFamily, ProtocolMetadata, ProtocolPayload,
    ProtocolPayloadBody, ProtocolResponseStatus, ProviderExecutionFailure,
    ProviderExecutionRequest, ProviderExecutionResult, ProviderExecutor, ResponseMode,
    RoutingAvailabilitySnapshot, RoutingAvailabilityState, RoutingCandidate, RoutingPolicy,
    RoutingTarget, RoutingTargetAvailability, StreamEvent, StreamTerminalState, StreamingResponse,
};

#[test]
fn valid_request_routes_executes_provider_and_serializes_response() -> Result<(), CoreError> {
    let recording_executor = RecordingExecutor::new(success_executor()?);
    let policy = policy_for(RoutingTarget::provider_account(
        "mock-openai",
        "acct-primary",
    ));
    let availability = availability_for(
        RoutingTarget::provider_account("mock-openai", "acct-primary"),
        RoutingAvailabilityState::Available,
    );

    let response = MinimalProxyEngine::execute(
        valid_request("smoke-model")?,
        MinimalProxyEngineConfig::new(&policy, &availability, &recording_executor),
    )?;

    assert_eq!(response.status_code, 200);
    assert_eq!(response.content_type, MINIMAL_PROXY_JSON_CONTENT_TYPE);
    assert!(response.body.contains(r#""object":"chat.completion""#));
    assert!(response.body.contains(r#""model":"smoke-model""#));
    assert!(response.body.contains("provider hello"));

    let recorded = recording_executor
        .take_recorded_request()
        .expect("recorded request");
    assert_eq!(recorded.provider_id, "mock-openai");
    assert_eq!(recorded.account_id.as_deref(), Some("acct-primary"));
    assert_eq!(recorded.request.protocol, ProtocolMetadata::open_ai());
    assert_eq!(recorded.request.model, "smoke-model");
    assert!(matches!(
        recorded.request.payload.body,
        ProtocolPayloadBody::Opaque(_)
    ));

    Ok(())
}

#[test]
fn invalid_request_fails_before_provider_execution() -> Result<(), CoreError> {
    let recording_executor = RecordingExecutor::new(success_executor()?);
    let policy = policy_for(RoutingTarget::provider_account(
        "mock-openai",
        "acct-primary",
    ));
    let availability = availability_for(
        RoutingTarget::provider_account("mock-openai", "acct-primary"),
        RoutingAvailabilityState::Available,
    );

    let error = MinimalProxyEngine::execute(
        MinimalProxyRequest::open_ai_chat_completions(
            br#"{"messages":[{"role":"user","content":"hi"}]}"#.to_vec(),
        )?,
        MinimalProxyEngineConfig::new(&policy, &availability, &recording_executor),
    )
    .expect_err("missing model must fail");

    assert!(matches!(
        error,
        CoreError::MinimalProxyRequestValidation {
            field: "model",
            code: MinimalProxyErrorCode::MissingModel,
            ..
        }
    ));
    assert!(recording_executor.take_recorded_request().is_none());

    Ok(())
}

#[test]
fn malformed_and_blank_model_errors_map_to_stable_400_json() -> Result<(), CoreError> {
    let recording_executor = RecordingExecutor::new(success_executor()?);
    let policy = policy_for(RoutingTarget::provider_account(
        "mock-openai",
        "acct-primary",
    ));
    let availability = availability_for(
        RoutingTarget::provider_account("mock-openai", "acct-primary"),
        RoutingAvailabilityState::Available,
    );

    let malformed = MinimalProxyEngine::execute_to_response(
        MinimalProxyRequest::open_ai_chat_completions(br#"{"#.to_vec())?,
        MinimalProxyEngineConfig::new(&policy, &availability, &recording_executor),
    );
    let blank = MinimalProxyEngine::execute_to_response(
        MinimalProxyRequest::open_ai_chat_completions(
            br#"{"model":"   ","messages":[{"role":"user","content":"hi"}]}"#.to_vec(),
        )?,
        MinimalProxyEngineConfig::new(&policy, &availability, &recording_executor),
    );

    assert_eq!(malformed.status_code, 400);
    assert!(malformed.body.contains(r#""code":"invalid_json""#));
    assert_eq!(blank.status_code, 400);
    assert!(blank.body.contains(r#""code":"blank_model""#));
    assert!(recording_executor.take_recorded_request().is_none());

    Ok(())
}

#[test]
fn provider_failure_maps_to_proxy_failure_response() -> Result<(), CoreError> {
    let executor = MockProviderHarness::new(
        "mock-openai",
        "Mock OpenAI",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Failed(ProviderExecutionFailure::failed_outcome(
            "provider_unavailable",
            "provider unavailable",
        )),
    )?
    .with_account(MockProviderAccount::new("acct-primary", "Primary account"));
    let policy = policy_for(RoutingTarget::provider_account(
        "mock-openai",
        "acct-primary",
    ));
    let availability = availability_for(
        RoutingTarget::provider_account("mock-openai", "acct-primary"),
        RoutingAvailabilityState::Available,
    );

    let error = MinimalProxyEngine::execute(
        valid_request("smoke-model")?,
        MinimalProxyEngineConfig::new(&policy, &availability, &executor),
    )
    .expect_err("provider failure must remain structured");
    assert!(matches!(
        &error,
        CoreError::ProviderExecution {
            provider_id,
            account_id: Some(account_id),
            failure: ProviderExecutionFailure::FailedOutcome { code, .. },
        } if provider_id == "mock-openai" && account_id == "acct-primary" && code == "provider_unavailable"
    ));

    let response = MinimalProxyResponseForTest::from_error(&error);
    assert_eq!(response.status_code, 502);
    assert!(
        response
            .body
            .contains(r#""code":"provider_execution_failed""#)
    );

    Ok(())
}

#[test]
fn unsupported_response_mode_maps_to_structured_failure() -> Result<(), CoreError> {
    let stream =
        StreamingResponse::new(vec![
            StreamEvent::Terminal(StreamTerminalState::completed()),
        ])?;
    let executor = MockProviderHarness::new(
        "mock-openai",
        "Mock OpenAI",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::SuccessWithMode {
            response_mode: ResponseMode::Streaming(stream),
            supports_streaming: true,
        },
    )?
    .with_account(MockProviderAccount::new("acct-primary", "Primary account"));
    let policy = policy_for(RoutingTarget::provider_account(
        "mock-openai",
        "acct-primary",
    ));
    let availability = availability_for(
        RoutingTarget::provider_account("mock-openai", "acct-primary"),
        RoutingAvailabilityState::Available,
    );

    let response = MinimalProxyEngine::execute_to_response(
        valid_request("smoke-model")?,
        MinimalProxyEngineConfig::new(&policy, &availability, &executor),
    );

    assert_eq!(response.status_code, 502);
    assert!(
        response
            .body
            .contains(r#""code":"unsupported_response_mode""#)
    );

    Ok(())
}

#[test]
fn routing_failure_prevents_provider_execution() -> Result<(), CoreError> {
    let recording_executor = RecordingExecutor::new(success_executor()?);
    let target = RoutingTarget::provider_account("mock-openai", "acct-primary");
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "smoke-model",
        vec![RoutingCandidate::new(target.clone())],
    )])
    .with_fallback(FallbackBehavior::new(true, false));
    let availability = availability_for(
        target,
        RoutingAvailabilityState::Unavailable {
            reason: "quota pressure".to_string(),
        },
    );

    let response = MinimalProxyEngine::execute_to_response(
        valid_request("smoke-model")?,
        MinimalProxyEngineConfig::new(&policy, &availability, &recording_executor),
    );

    assert_eq!(response.status_code, 502);
    assert!(response.body.contains(r#""code":"routing_failed""#));
    assert!(recording_executor.take_recorded_request().is_none());

    Ok(())
}

struct MinimalProxyResponseForTest;

impl MinimalProxyResponseForTest {
    fn from_error(error: &CoreError) -> oxmux::MinimalProxyResponse {
        oxmux::MinimalProxyResponse::from_core_error(error)
    }
}

struct RecordingExecutor {
    inner: MockProviderHarness,
    recorded_request: RefCell<Option<ProviderExecutionRequest>>,
}

impl RecordingExecutor {
    fn new(inner: MockProviderHarness) -> Self {
        Self {
            inner,
            recorded_request: RefCell::new(None),
        }
    }

    fn take_recorded_request(&self) -> Option<ProviderExecutionRequest> {
        self.recorded_request.borrow_mut().take()
    }
}

impl ProviderExecutor for RecordingExecutor {
    fn execute(
        &self,
        request: ProviderExecutionRequest,
    ) -> Result<ProviderExecutionResult, CoreError> {
        *self.recorded_request.borrow_mut() = Some(request.clone());
        self.inner.execute(request)
    }
}

fn valid_request(model: &str) -> Result<MinimalProxyRequest, CoreError> {
    MinimalProxyRequest::open_ai_chat_completions(
        format!(r#"{{"model":"{model}","messages":[{{"role":"user","content":"hi"}}]}}"#)
            .into_bytes(),
    )
}

fn success_executor() -> Result<MockProviderHarness, CoreError> {
    MockProviderHarness::new(
        "mock-openai",
        "Mock OpenAI",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Success(CanonicalProtocolResponse::new(
            ProtocolMetadata::open_ai(),
            ProtocolResponseStatus::success(),
            ProtocolPayload::opaque("application/json", b"provider hello".to_vec()),
        )?),
    )
    .map(|executor| {
        executor.with_account(MockProviderAccount::new("acct-primary", "Primary account"))
    })
}

fn policy_for(target: RoutingTarget) -> RoutingPolicy {
    RoutingPolicy::new(vec![ModelRoute::new(
        "smoke-model",
        vec![RoutingCandidate::new(target)],
    )])
}

fn availability_for(
    target: RoutingTarget,
    state: RoutingAvailabilityState,
) -> RoutingAvailabilitySnapshot {
    RoutingAvailabilitySnapshot::new(vec![RoutingTargetAvailability::new(target, state)])
}
