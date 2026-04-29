use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use oxmux::{
    AccountSummary, AuthMethodCategory, AuthState, BoundEndpoint, ConfigurationSnapshot,
    ConfigurationUpdateIntent, CoreError, CoreHealthState, DegradedReason, LastCheckedMetadata,
    LifecycleControlIntent, ManagementSnapshot, MeteredValue, ProtocolFamily, ProtocolMetadata,
    ProtocolPayload, ProviderCapability, ProviderSummary, ProxyLifecycleState, QuotaState,
    QuotaSummary, ResponseMode, RoutingDefault, StreamEvent, StreamMetadata, StreamTerminalState,
    StreamingResponse, UptimeMetadata, UsageSummary, core_identity,
};

#[test]
fn core_can_be_used_directly() {
    let identity = core_identity();

    assert_eq!(identity.crate_name, "oxmux");
    assert_eq!(identity.version, "0.1.0");
}

#[test]
fn streaming_primitives_are_usable_through_public_facade() -> Result<(), CoreError> {
    let stream = StreamingResponse::new(vec![
        StreamEvent::Metadata(StreamMetadata::new("provider", "mock")?),
        StreamEvent::Terminal(StreamTerminalState::completed()),
    ])?;
    let mode = ResponseMode::Streaming(stream.clone());

    assert!(mode.complete_response().is_none());
    assert_eq!(mode.streaming_response(), Some(&stream));
    assert_eq!(ProtocolMetadata::open_ai().family(), ProtocolFamily::OpenAi);
    assert!(matches!(ProtocolPayload::empty(), ProtocolPayload { .. }));

    Ok(())
}

#[test]
fn management_snapshot_can_be_constructed_from_in_memory_values() {
    let endpoint = BoundEndpoint {
        socket_addr: SocketAddr::from(([127, 0, 0, 1], 8787)),
    };
    let uptime = UptimeMetadata {
        started_at_unix_seconds: 1_700_000_000,
        elapsed: Duration::from_secs(42),
    };
    let degraded_reason = DegradedReason {
        component: "provider:openai".to_string(),
        message: "quota has not been refreshed".to_string(),
    };
    let account = AccountSummary {
        account_id: "acct-primary".to_string(),
        display_name: "Primary account".to_string(),
        auth_state: AuthState::CredentialReference {
            reference_name: "desktop-secret-ref".to_string(),
        },
        quota_state: QuotaState::Degraded {
            remaining: None,
            reason: "quota fetch deferred".to_string(),
        },
        last_checked: Some(LastCheckedMetadata {
            unix_timestamp_seconds: 1_700_000_010,
            age_seconds: 12,
        }),
        degraded_reasons: vec![degraded_reason.clone()],
    };
    let provider = ProviderSummary {
        provider_id: "openai".to_string(),
        display_name: "OpenAI".to_string(),
        capabilities: vec![ProviderCapability {
            protocol_family: ProtocolFamily::OpenAi,
            supports_streaming: true,
            auth_method: AuthMethodCategory::ApiKey,
            routing_eligible: true,
        }],
        accounts: vec![account],
        degraded_reasons: vec![degraded_reason.clone()],
    };

    let snapshot = ManagementSnapshot {
        identity: core_identity(),
        lifecycle: ProxyLifecycleState::Degraded {
            endpoint: Some(endpoint),
            uptime: Some(uptime),
            reasons: vec![degraded_reason.clone()],
        },
        health: CoreHealthState::Degraded {
            reasons: vec![degraded_reason],
        },
        configuration: ConfigurationSnapshot {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8787,
            auto_start: false,
            logging_enabled: true,
            usage_collection_enabled: false,
            routing_default: RoutingDefault::named("manual"),
            provider_references: vec!["openai".to_string()],
        },
        file_configuration: None,
        last_configuration_load_failure: None,
        providers: vec![provider],
        usage: UsageSummary {
            requests: MeteredValue::Known(3),
            input_tokens: MeteredValue::Known(120),
            output_tokens: MeteredValue::Known(80),
            model_totals: MeteredValue::Known(1),
            provider_totals: MeteredValue::Known(1),
            account_totals: MeteredValue::Known(1),
        },
        quota: QuotaSummary {
            requests: QuotaState::Unknown,
            tokens: QuotaState::Unavailable {
                reason: "provider quota endpoint is not implemented".to_string(),
            },
        },
        warnings: vec!["quota data is placeholder-only".to_string()],
        errors: vec![CoreError::UsageQuotaSummary {
            message: "quota fetch deferred".to_string(),
        }],
    };

    assert_eq!(snapshot.identity.crate_name, "oxmux");
    assert_eq!(snapshot.providers.len(), 1);
    assert_eq!(snapshot.warnings, ["quota data is placeholder-only"]);
}

#[test]
fn lifecycle_states_and_intents_are_inert_descriptions() {
    let running = ProxyLifecycleState::Running {
        endpoint: BoundEndpoint {
            socket_addr: SocketAddr::from(([127, 0, 0, 1], 8787)),
        },
        uptime: UptimeMetadata {
            started_at_unix_seconds: 1_700_000_000,
            elapsed: Duration::from_secs(5),
        },
    };

    assert!(matches!(running, ProxyLifecycleState::Running { .. }));
    assert_eq!(
        LifecycleControlIntent::Start.validate(),
        Ok(LifecycleControlIntent::Start)
    );
    assert_eq!(
        LifecycleControlIntent::Stop.validate(),
        Ok(LifecycleControlIntent::Stop)
    );
    assert_eq!(
        LifecycleControlIntent::Restart.validate(),
        Ok(LifecycleControlIntent::Restart)
    );
    assert_eq!(
        LifecycleControlIntent::RefreshStatus.validate(),
        Ok(LifecycleControlIntent::RefreshStatus)
    );
}

#[test]
fn configuration_update_validation_reports_structured_errors() {
    let valid = ConfigurationUpdateIntent {
        listen_address: Some("127.0.0.1".to_string()),
        port: Some(8787),
        auto_start: Some(true),
        logging_enabled: Some(true),
        usage_collection_enabled: Some(false),
        routing_default: Some(RoutingDefault::named("primary")),
        provider_references: Some(vec!["openai".to_string()]),
    };

    let validated = valid.validate();
    assert!(validated.is_ok());

    let invalid_listen_address = ConfigurationUpdateIntent {
        listen_address: Some("localhost".to_string()),
        ..ConfigurationUpdateIntent::default()
    };
    assert!(matches!(
        invalid_listen_address.validate(),
        Err(CoreError::ConfigurationValidation {
            field: "listen_address",
            ..
        })
    ));

    let invalid_port = ConfigurationUpdateIntent {
        port: Some(0),
        ..ConfigurationUpdateIntent::default()
    };
    assert!(matches!(
        invalid_port.validate(),
        Err(CoreError::ConfigurationValidation { field: "port", .. })
    ));

    let invalid_routing = ConfigurationUpdateIntent {
        routing_default: Some(RoutingDefault::named("   ")),
        ..ConfigurationUpdateIntent::default()
    };
    assert!(matches!(
        invalid_routing.validate(),
        Err(CoreError::ConfigurationValidation {
            field: "routing_default",
            ..
        })
    ));

    let invalid_provider_reference = ConfigurationUpdateIntent {
        provider_references: Some(vec!["openai".to_string(), " ".to_string()]),
        ..ConfigurationUpdateIntent::default()
    };
    assert!(matches!(
        invalid_provider_reference.validate(),
        Err(CoreError::ConfigurationValidation {
            field: "provider_references",
            ..
        })
    ));
}

#[test]
fn provider_account_usage_and_quota_summaries_distinguish_placeholder_states() {
    let provider = ProviderSummary {
        provider_id: "anthropic".to_string(),
        display_name: "Anthropic".to_string(),
        capabilities: vec![ProviderCapability {
            protocol_family: ProtocolFamily::Claude,
            supports_streaming: true,
            auth_method: AuthMethodCategory::OAuth,
            routing_eligible: false,
        }],
        accounts: vec![AccountSummary {
            account_id: "acct-unknown".to_string(),
            display_name: "Unknown quota account".to_string(),
            auth_state: AuthState::Unknown,
            quota_state: QuotaState::Unknown,
            last_checked: None,
            degraded_reasons: Vec::new(),
        }],
        degraded_reasons: vec![DegradedReason {
            component: "routing".to_string(),
            message: "routing is disabled until configuration is complete".to_string(),
        }],
    };
    let zero_usage = UsageSummary::zero();
    let unknown_usage = UsageSummary::unknown();
    let quota = QuotaSummary {
        requests: QuotaState::Unavailable {
            reason: "not connected".to_string(),
        },
        tokens: QuotaState::Degraded {
            remaining: Some(100),
            reason: "stale provider data".to_string(),
        },
    };

    assert_eq!(provider.accounts[0].quota_state, QuotaState::Unknown);
    assert_eq!(zero_usage.requests, MeteredValue::Zero);
    assert_eq!(unknown_usage.requests, MeteredValue::Unknown);
    assert!(matches!(quota.requests, QuotaState::Unavailable { .. }));
    assert!(matches!(quota.tokens, QuotaState::Degraded { .. }));
}
