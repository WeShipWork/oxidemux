//! Integration tests for file-backed oxmux configuration.

use std::error::Error;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oxmux::{
    AutoStartIntent, ConfigurationBoundary, ConfigurationErrorKind, ConfigurationSourceMetadata,
    CoreError, FileConfigurationState, InvalidConfigurationValue, LayeredConfigurationInput,
    LayeredConfigurationReloadOutcome, LayeredConfigurationState, LoggingSetting,
    ManagementSnapshot, ProtocolFamily, QuotaState, RoutingTarget, StreamingCancellationPolicy,
    ValidatedFileConfiguration,
};

const VALID_TOML: &str = include_str!("fixtures/file_configuration/valid.toml");

#[test]
fn valid_toml_loads_typed_configuration() -> Result<(), Box<dyn Error>> {
    let configuration = ConfigurationBoundary::load_contents(VALID_TOML)?;

    assert_eq!(
        configuration.proxy.listen_address,
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    );
    assert_eq!(configuration.proxy.port, 8787);
    assert_eq!(configuration.providers.len(), 2);
    assert_eq!(configuration.providers[0].id, "mock-openai");
    assert_eq!(
        configuration.providers[0].protocol_family,
        ProtocolFamily::OpenAi
    );
    assert!(configuration.providers[0].accounts[0].credential_reference_present());
    assert_eq!(configuration.routing_defaults.len(), 3);
    assert_eq!(configuration.routing_default_groups.len(), 2);
    assert_eq!(configuration.routing_default_groups[0].name, "chat");
    assert_eq!(configuration.routing_default_groups[0].model, "gpt-4o-mini");
    assert_eq!(configuration.routing_default_groups[0].candidates.len(), 2);
    assert!(configuration.routing_default_groups[0].candidates[0].fallback_enabled);
    assert!(!configuration.routing_default_groups[0].candidates[1].fallback_enabled);
    assert_eq!(configuration.routing_defaults[0].name, "chat");
    assert_eq!(configuration.routing_defaults[0].model, "gpt-4o-mini");
    assert_eq!(
        configuration.routing_defaults[2].target,
        RoutingTarget::provider("mock-openai")
    );
    assert_eq!(configuration.logging, LoggingSetting::Standard);
    assert!(configuration.usage_collection_enabled);
    assert_eq!(configuration.auto_start, AutoStartIntent::Disabled);
    assert!(configuration.streaming.is_disabled());
    assert!(configuration.warnings.is_empty());
    Ok(())
}

#[test]
fn streaming_policy_toml_loads_typed_configuration() -> Result<(), Box<dyn Error>> {
    let configuration = ValidatedFileConfiguration::load_contents(&with_streaming(
        r#"
[streaming]
keepalive-interval-ms = 15000
bootstrap-retry-count = 2
timeout-ms = 120000
cancellation = "client-disconnect"
"#,
    ))?;

    assert_eq!(configuration.streaming.keepalive_interval_ms, Some(15_000));
    assert_eq!(configuration.streaming.bootstrap_retry_count, 2);
    assert_eq!(configuration.streaming.timeout_ms, Some(120_000));
    assert_eq!(
        configuration.streaming.cancellation,
        StreamingCancellationPolicy::ClientDisconnect
    );
    assert_eq!(
        configuration.configuration_snapshot().streaming,
        configuration.streaming
    );
    Ok(())
}

#[test]
fn streaming_policy_partial_defaults_and_timeout_metadata_policy_load() -> Result<(), Box<dyn Error>>
{
    let configuration = ValidatedFileConfiguration::load_contents(&with_streaming(
        r#"
[streaming]
timeout-ms = 30000
"#,
    ))?;

    assert_eq!(configuration.streaming.keepalive_interval_ms, None);
    assert_eq!(configuration.streaming.bootstrap_retry_count, 0);
    assert_eq!(configuration.streaming.timeout_ms, Some(30_000));
    assert_eq!(
        configuration.streaming.cancellation,
        StreamingCancellationPolicy::Disabled
    );
    Ok(())
}

#[test]
fn streaming_policy_validation_reports_structured_errors() {
    for (toml, kind, field, invalid_value) in [
        (
            with_streaming("[streaming]\nkeepalive-interval-ms = 0\n"),
            ConfigurationErrorKind::InvalidStreamingKeepaliveInterval,
            "streaming.keepalive-interval-ms",
            InvalidConfigurationValue::OutOfRange,
        ),
        (
            with_streaming("[streaming]\ntimeout-ms = -1\n"),
            ConfigurationErrorKind::InvalidStreamingTimeout,
            "streaming.timeout-ms",
            InvalidConfigurationValue::OutOfRange,
        ),
        (
            with_streaming("[streaming]\nbootstrap-retry-count = 11\n"),
            ConfigurationErrorKind::InvalidStreamingBootstrapRetryCount,
            "streaming.bootstrap-retry-count",
            InvalidConfigurationValue::OutOfRange,
        ),
        (
            with_streaming("[streaming]\nbootstrap-retry-count = 1.5\n"),
            ConfigurationErrorKind::InvalidStreamingBootstrapRetryCount,
            "streaming.bootstrap-retry-count",
            InvalidConfigurationValue::Malformed,
        ),
        (
            with_streaming("[streaming]\ntimeout-ms = \"fast\"\n"),
            ConfigurationErrorKind::InvalidStreamingTimeout,
            "streaming.timeout-ms",
            InvalidConfigurationValue::Malformed,
        ),
        (
            with_streaming("[streaming]\ncancellation = \"socket-close\"\n"),
            ConfigurationErrorKind::InvalidStreamingCancellation,
            "streaming.cancellation",
            InvalidConfigurationValue::Unsupported,
        ),
        (
            with_streaming("[streaming]\ncancellation = \"timeout\"\n"),
            ConfigurationErrorKind::InvalidStreamingCancellation,
            "streaming.cancellation",
            InvalidConfigurationValue::Missing,
        ),
    ] {
        assert_error_value(
            ValidatedFileConfiguration::load_contents(&toml),
            kind,
            field,
            invalid_value,
        );
    }

    assert_error_kind(
        ValidatedFileConfiguration::load_contents(&with_streaming(
            "[streaming]\nunknown-streaming-field = true\n",
        )),
        ConfigurationErrorKind::UnknownField,
        "streaming.unknown-streaming-field",
    );
}

#[test]
fn path_loading_reports_read_parse_and_unsupported_format_errors() -> Result<(), Box<dyn Error>> {
    let directory = unique_temp_dir()?;
    fs::create_dir_all(&directory)?;
    let valid_path = directory.join("config.toml");
    fs::write(&valid_path, VALID_TOML)?;

    let loaded = ValidatedFileConfiguration::load_file(&valid_path)?;
    assert_eq!(loaded.source.path, Some(valid_path.clone()));

    let unsupported_path = directory.join("config.json");
    assert_error_kind(
        ValidatedFileConfiguration::load_file(&unsupported_path),
        ConfigurationErrorKind::UnsupportedFormat,
        "source.path",
    );

    let missing_path = directory.join("missing.toml");
    assert_error_kind(
        ValidatedFileConfiguration::load_file(&missing_path),
        ConfigurationErrorKind::ReadFailed,
        "source.path",
    );

    let parse_path = directory.join("broken.toml");
    fs::write(&parse_path, "version = 1\n[proxy\n")?;
    assert_error_kind(
        ValidatedFileConfiguration::load_file(&parse_path),
        ConfigurationErrorKind::ParseFailed,
        "configuration",
    );

    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn validation_reports_structured_semantic_errors() {
    for (toml, kind, field) in [
        (
            with_replace(
                "listen-address = \"127.0.0.1\"",
                "listen-address = \"localhost\"",
            ),
            ConfigurationErrorKind::InvalidListenAddress,
            "proxy.listen-address",
        ),
        (
            with_replace(
                "listen-address = \"127.0.0.1\"",
                "listen-address = \"0.0.0.0\"",
            ),
            ConfigurationErrorKind::InvalidListenAddress,
            "proxy.listen-address",
        ),
        (
            with_replace("port = 8787", "port = 0"),
            ConfigurationErrorKind::InvalidPort,
            "proxy.port",
        ),
        (
            with_replace("port = 8787", "port = 70000"),
            ConfigurationErrorKind::InvalidPort,
            "proxy.port",
        ),
        (
            with_replace("protocol-family = \"openai\"", "protocol-family = \"smtp\""),
            ConfigurationErrorKind::InvalidProviderProtocolFamily,
            "providers[0].protocol-family",
        ),
        (
            with_replace(
                "credential-reference = \"mock-openai/default\"",
                "credential-reference = \"sk-secret-token\"",
            ),
            ConfigurationErrorKind::InvalidCredentialReference,
            "providers[0].accounts[0].credential-reference",
        ),
        (
            with_replace("routing-eligible = true", "routing-eligible = false"),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[0].provider-id",
        ),
        (
            with_replace("fallback-enabled = true", "fallback-enabled = false"),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[0].fallback-enabled",
        ),
        (
            with_replace(
                "provider-id = \"mock-openai\"",
                "provider-id = \"missing-provider\"",
            ),
            ConfigurationErrorKind::UnknownProviderReference,
            "routing.defaults[0].provider-id",
        ),
        (
            with_replace(
                "account-id = \"default\"",
                "account-id = \"missing-account\"",
            ),
            ConfigurationErrorKind::UnknownAccountReference,
            "routing.defaults[0].account-id",
        ),
        (
            with_replace("name = \"chat\"", "name = \" \""),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[0].name",
        ),
        (
            format!(
                "{VALID_TOML}\n[[routing.defaults]]\nname = \"chat\"\nmodel = \"gpt-4o-mini\"\nprovider-id = \"mock-openai\"\naccount-id = \"default\"\nfallback-enabled = true\n"
            ),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[3].provider-id",
        ),
        (
            duplicate_provider_toml(),
            ConfigurationErrorKind::DuplicateProviderId,
            "providers[2].id",
        ),
        (
            duplicate_account_toml(),
            ConfigurationErrorKind::DuplicateAccountId,
            "providers[0].accounts[1].id",
        ),
        (
            with_replace("logging = \"standard\"", "logging = \"trace\""),
            ConfigurationErrorKind::InvalidLoggingSetting,
            "observability.logging",
        ),
        (
            with_replace(
                "usage-collection = true",
                "usage-collection = \"sometimes\"",
            ),
            ConfigurationErrorKind::InvalidUsageCollectionSetting,
            "observability.usage-collection",
        ),
        (
            with_replace("auto-start = \"disabled\"", "auto-start = \"login-item\""),
            ConfigurationErrorKind::InvalidAutoStartIntent,
            "lifecycle.auto-start",
        ),
    ] {
        assert_error_kind(
            ValidatedFileConfiguration::load_contents(&toml),
            kind,
            field,
        );
    }
}

#[test]
fn management_snapshot_reflects_valid_file_configuration_without_verified_health()
-> Result<(), Box<dyn Error>> {
    let mut state = FileConfigurationState::new();
    state.replace_from_contents_with_source(
        VALID_TOML,
        ConfigurationSourceMetadata::for_path("/tmp/oxidemux/config.toml"),
    )?;

    let snapshot = ManagementSnapshot::from_file_configuration_state(&state);
    let Some(file_configuration) = snapshot.file_configuration else {
        return Err("expected active file configuration".into());
    };

    assert_eq!(snapshot.configuration.port, 8787);
    assert_eq!(
        file_configuration.source.description,
        "/tmp/oxidemux/config.toml"
    );
    assert_eq!(file_configuration.logging, LoggingSetting::Standard);
    assert!(file_configuration.usage_collection_enabled);
    assert_eq!(file_configuration.auto_start, AutoStartIntent::Disabled);
    assert_eq!(snapshot.providers.len(), 2);
    assert!(matches!(
        snapshot.providers[0].accounts[0].auth_state,
        oxmux::AuthState::CredentialReference { .. }
    ));
    assert_eq!(
        snapshot.providers[0].accounts[0].quota_state,
        QuotaState::Unknown
    );
    assert!(snapshot.providers[0].accounts[0].last_checked.is_none());
    assert!(snapshot.providers[0].degraded_reasons.is_empty());
    assert!(snapshot.last_configuration_load_failure.is_none());
    assert!(snapshot.errors.is_empty());
    Ok(())
}

#[test]
fn replacement_hooks_preserve_last_valid_state_and_clear_failure_metadata()
-> Result<(), Box<dyn Error>> {
    let invalid = with_replace(
        "listen-address = \"127.0.0.1\"",
        "listen-address = \"8.8.8.8\"",
    );
    let mut state = FileConfigurationState::new();

    assert_error_kind(
        state.replace_from_contents(&invalid),
        ConfigurationErrorKind::InvalidListenAddress,
        "proxy.listen-address",
    );
    assert!(state.active().is_none());
    assert!(state.last_failure().is_some());

    state.replace_from_contents(VALID_TOML)?;
    assert!(state.active().is_some());
    assert!(state.last_failure().is_none());

    let active_before_failure = match state.active() {
        Some(configuration) => configuration.clone(),
        None => return Err("expected active configuration".into()),
    };
    assert_error_kind(
        state.replace_from_contents(&invalid),
        ConfigurationErrorKind::InvalidListenAddress,
        "proxy.listen-address",
    );
    assert_eq!(state.active(), Some(&active_before_failure));
    assert!(state.last_failure().is_some());

    state.replace_from_contents(&with_replace("port = 8787", "port = 8788"))?;
    let snapshot = ManagementSnapshot::from_file_configuration_state(&state);
    assert_eq!(snapshot.configuration.port, 8788);
    assert!(snapshot.last_configuration_load_failure.is_none());
    assert!(snapshot.errors.is_empty());
    Ok(())
}

#[test]
fn fixture_error_cases_cover_schema_and_reference_failures() {
    for (toml, kind, field) in [
        (
            include_str!("fixtures/file_configuration/unknown-field.toml").to_string(),
            ConfigurationErrorKind::UnknownField,
            "lifecycle.extra-field",
        ),
        (
            with_replace("version = 1", "version = 2"),
            ConfigurationErrorKind::InvalidVersion,
            "version",
        ),
        (
            with_replace("listen-address = \"127.0.0.1\"", ""),
            ConfigurationErrorKind::MissingRequiredField,
            "proxy.listen-address",
        ),
        (
            duplicate_provider_toml(),
            ConfigurationErrorKind::DuplicateProviderId,
            "providers[2].id",
        ),
        (
            duplicate_account_toml(),
            ConfigurationErrorKind::DuplicateAccountId,
            "providers[0].accounts[1].id",
        ),
        (
            with_replace(
                "protocol-family = \"openai\"",
                "protocol-family = \"unknown\"",
            ),
            ConfigurationErrorKind::InvalidProviderProtocolFamily,
            "providers[0].protocol-family",
        ),
        (
            with_replace(
                "credential-reference = \"mock-openai/default\"",
                "credential-reference = \"secret-token\"",
            ),
            ConfigurationErrorKind::InvalidCredentialReference,
            "providers[0].accounts[0].credential-reference",
        ),
        (
            with_replace("provider-id = \"mock-openai\"", "provider-id = \"unknown\""),
            ConfigurationErrorKind::UnknownProviderReference,
            "routing.defaults[0].provider-id",
        ),
        (
            with_replace("account-id = \"default\"", "account-id = \"unknown\""),
            ConfigurationErrorKind::UnknownAccountReference,
            "routing.defaults[0].account-id",
        ),
        (
            with_replace("model = \"gpt-4o-mini\"", "model = \" \""),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[0].model",
        ),
        (
            format!(
                "{VALID_TOML}\n[[routing.defaults]]\nname = \"chat\"\nmodel = \"gpt-4o-mini\"\nprovider-id = \"mock-openai\"\naccount-id = \"default\"\nfallback-enabled = false\n"
            ),
            ConfigurationErrorKind::InvalidRoutingDefault,
            "routing.defaults[3].provider-id",
        ),
    ] {
        assert_error_kind(
            ValidatedFileConfiguration::load_contents(&toml),
            kind,
            field,
        );
    }

    assert_error_kind(
        ValidatedFileConfiguration::load_file("config.yaml"),
        ConfigurationErrorKind::UnsupportedFormat,
        "source.path",
    );
    assert_error_kind(
        ValidatedFileConfiguration::load_contents(include_str!(
            "fixtures/file_configuration/parse-failure.toml"
        )),
        ConfigurationErrorKind::ParseFailed,
        "configuration",
    );
    assert_error_value(
        ValidatedFileConfiguration::load_contents(&with_replace(
            "credential-reference = \"mock-openai/default\"",
            "credential-reference = \"mock-openai/api_key\"",
        )),
        ConfigurationErrorKind::InvalidCredentialReference,
        "providers[0].accounts[0].credential-reference",
        InvalidConfigurationValue::SecretLike,
    );
}

#[test]
fn layered_defaults_fill_missing_user_fields_and_user_scalars_override()
-> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let user = r#"
[proxy]
port = 9797

[observability]
usage-collection = false

[lifecycle]
auto-start = "enabled"
"#;

    let outcome = state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            user,
            ConfigurationSourceMetadata::for_path("user.toml"),
        ),
    ]);

    assert!(matches!(
        outcome,
        LayeredConfigurationReloadOutcome::Replaced { .. }
    ));
    let Some(active) = state.active() else {
        return Err("expected active layered configuration".into());
    };
    assert_eq!(
        active.configuration.proxy.listen_address,
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    );
    assert_eq!(active.configuration.proxy.port, 9797);
    assert!(!active.configuration.usage_collection_enabled);
    assert_eq!(active.configuration.auto_start, AutoStartIntent::Enabled);
    assert_eq!(active.configuration.providers.len(), 2);
    Ok(())
}

#[test]
fn layered_provider_accounts_merge_deterministically_and_routes_replace()
-> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let user = r#"
[[providers]]
id = "mock-openai"
protocol-family = "openai"
routing-eligible = true

[[providers.accounts]]
id = "secondary"
credential-reference = "mock-openai/secondary"

[[providers]]
id = "mock-gemini"
protocol-family = "gemini"
routing-eligible = true

[[providers.accounts]]
id = "default"
credential-reference = "mock-gemini/default"

[[routing.defaults]]
name = "chat"
model = "gemini-1.5-pro"
provider-id = "mock-gemini"
account-id = "default"
fallback-enabled = false
"#;

    let outcome = state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            user,
            ConfigurationSourceMetadata::for_path("user.toml"),
        ),
    ]);

    assert!(matches!(
        outcome,
        LayeredConfigurationReloadOutcome::Replaced { .. }
    ));
    let configuration = &state
        .active()
        .ok_or("expected active layered configuration")?
        .configuration;
    assert_eq!(configuration.providers.len(), 3);
    assert_eq!(configuration.providers[0].id, "mock-claude");
    assert_eq!(configuration.providers[1].id, "mock-gemini");
    assert_eq!(configuration.providers[2].id, "mock-openai");
    assert_eq!(configuration.providers[2].accounts.len(), 2);
    assert_eq!(configuration.providers[2].accounts[0].id, "default");
    assert_eq!(configuration.providers[2].accounts[1].id, "secondary");
    assert_eq!(configuration.routing_defaults.len(), 1);
    assert_eq!(configuration.routing_defaults[0].model, "gemini-1.5-pro");
    assert_eq!(
        configuration.routing_defaults[0].target,
        RoutingTarget::provider_account("mock-gemini", "default")
    );
    Ok(())
}

#[test]
fn layered_streaming_policy_user_scalars_override_defaults() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let defaults =
        with_streaming("[streaming]\nkeepalive-interval-ms = 15000\nbootstrap-retry-count = 1\n");
    let user = r#"
[streaming]
timeout-ms = 45000
cancellation = "timeout"
"#;

    let outcome = state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(defaults),
        LayeredConfigurationInput::user_owned(
            user,
            ConfigurationSourceMetadata::for_path("user.toml"),
        ),
    ]);

    assert!(matches!(
        outcome,
        LayeredConfigurationReloadOutcome::Replaced { .. }
    ));
    let configuration = &state
        .active()
        .ok_or("expected active layered configuration")?
        .configuration;
    assert_eq!(configuration.streaming.keepalive_interval_ms, Some(15_000));
    assert_eq!(configuration.streaming.bootstrap_retry_count, 1);
    assert_eq!(configuration.streaming.timeout_ms, Some(45_000));
    assert_eq!(
        configuration.streaming.cancellation,
        StreamingCancellationPolicy::Timeout
    );
    Ok(())
}

#[test]
fn layered_rejected_candidates_preserve_prior_active_configuration() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let first = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    let active_fingerprint = match first {
        LayeredConfigurationReloadOutcome::Replaced {
            active_fingerprint, ..
        } => active_fingerprint,
        other => return Err(format!("expected replaced outcome, got {other:?}").into()),
    };
    let active_before_failure = state.active().ok_or("expected active config")?.clone();

    let invalid = r#"
[proxy]
listen-address = "8.8.8.8"
"#;
    let outcome = state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            invalid,
            ConfigurationSourceMetadata::for_path("broken.toml"),
        ),
    ]);

    let LayeredConfigurationReloadOutcome::Rejected(rejected) = outcome else {
        return Err("expected rejected outcome".into());
    };
    assert_eq!(
        rejected.previous_active_fingerprint,
        Some(active_fingerprint)
    );
    assert!(rejected.errors.iter().any(|error| {
        error.kind == ConfigurationErrorKind::InvalidListenAddress
            && error.field_path == "proxy.listen-address"
    }));
    assert_eq!(state.active(), Some(&active_before_failure));
    assert!(state.failed_candidate().is_some());
    Ok(())
}

#[test]
fn layered_equivalent_runtime_fingerprint_returns_unchanged() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let first = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    let active_fingerprint = match first {
        LayeredConfigurationReloadOutcome::Replaced {
            active_fingerprint, ..
        } => active_fingerprint,
        other => return Err(format!("expected replaced outcome, got {other:?}").into()),
    };

    let reformatted =
        format!("# comments and whitespace do not affect effective runtime\n\n{VALID_TOML}\n");
    let second = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        reformatted,
    )]);

    assert_eq!(
        second,
        LayeredConfigurationReloadOutcome::Unchanged {
            active_fingerprint,
            sources: vec![oxmux::ConfigurationLayerSource {
                kind: oxmux::ConfigurationLayerKind::BundledDefaults,
                source: ConfigurationSourceMetadata::memory(),
            }],
        }
    );
    Ok(())
}

#[test]
fn layered_management_snapshot_exposes_metadata_and_clears_failed_candidate()
-> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            "[proxy]\nlisten-address = \"8.8.8.8\"\n",
            ConfigurationSourceMetadata::for_path("broken.toml"),
        ),
    ]);

    let failed_snapshot = ManagementSnapshot::from_layered_configuration_state(&state);
    assert!(failed_snapshot.layered_configuration.is_some());
    assert!(failed_snapshot.last_layered_configuration_failure.is_some());
    assert_eq!(failed_snapshot.configuration.port, 8787);
    assert!(
        failed_snapshot
            .errors
            .iter()
            .any(|error| matches!(error, CoreError::Configuration { .. }))
    );

    state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            "[proxy]\nport = 9797\n",
            ConfigurationSourceMetadata::for_path("user.toml"),
        ),
    ]);
    let successful_snapshot = ManagementSnapshot::from_layered_configuration_state(&state);
    let layered = successful_snapshot
        .layered_configuration
        .ok_or("expected layered metadata")?;
    assert_eq!(successful_snapshot.configuration.port, 9797);
    assert_eq!(layered.sources.len(), 2);
    assert!(
        successful_snapshot
            .last_layered_configuration_failure
            .is_none()
    );
    assert!(successful_snapshot.errors.is_empty());
    assert!(matches!(
        successful_snapshot.providers[0].accounts[0].auth_state,
        oxmux::AuthState::CredentialReference { .. }
    ));
    assert_eq!(
        successful_snapshot.providers[0].accounts[0].quota_state,
        QuotaState::Unknown
    );
    Ok(())
}

#[test]
fn layered_credential_reference_changes_replace_without_leaking_management_values()
-> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let first = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    assert!(matches!(
        first,
        LayeredConfigurationReloadOutcome::Replaced { .. }
    ));

    let changed_credential = with_replace(
        "credential-reference = \"mock-openai/default\"",
        "credential-reference = \"mock-openai/rotated\"",
    );
    let second = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        changed_credential,
    )]);
    assert!(matches!(
        second,
        LayeredConfigurationReloadOutcome::Replaced { .. }
    ));

    let snapshot = ManagementSnapshot::from_layered_configuration_state(&state);
    assert!(format!("{snapshot:?}").contains("configured"));
    assert!(!format!("{snapshot:?}").contains("mock-openai/rotated"));
    Ok(())
}

#[test]
fn initial_invalid_layered_load_has_no_active_configuration() {
    let mut state = LayeredConfigurationState::new();
    let outcome = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        "version = 1\n[proxy]\nlisten-address = \"8.8.8.8\"\nport = 8787\n",
    )]);

    assert!(matches!(
        outcome,
        LayeredConfigurationReloadOutcome::Rejected(_)
    ));
    assert!(state.active().is_none());
    assert!(state.failed_candidate().is_some());
}

#[test]
fn layered_parse_rejects_unknown_fields_with_source_metadata() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let outcome = state.replace(vec![LayeredConfigurationInput::user_owned(
        "version = 1\nunknown-field = true\n",
        ConfigurationSourceMetadata::for_path("unknown.toml"),
    )]);

    let LayeredConfigurationReloadOutcome::Rejected(rejected) = outcome else {
        return Err("expected rejected outcome".into());
    };
    assert!(rejected.errors.iter().any(|error| {
        error.kind == ConfigurationErrorKind::UnknownField
            && error
                .source
                .as_ref()
                .is_some_and(|source| source.description == "unknown.toml")
    }));
    assert!(state.active().is_none());
    Ok(())
}

#[test]
fn layered_secret_like_credentials_are_rejected() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    let secret_like = with_replace(
        "credential-reference = \"mock-openai/default\"",
        "credential-reference = \"sk-secret-token\"",
    );
    let outcome = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        secret_like,
    )]);

    let LayeredConfigurationReloadOutcome::Rejected(rejected) = outcome else {
        return Err("expected rejected outcome".into());
    };
    assert!(rejected.errors.iter().any(|error| {
        error.kind == ConfigurationErrorKind::InvalidCredentialReference
            && error.invalid_value == InvalidConfigurationValue::SecretLike
    }));
    assert!(
        !rejected
            .errors
            .iter()
            .any(|error| error.kind == ConfigurationErrorKind::UnknownAccountReference)
    );
    assert!(state.active().is_none());
    Ok(())
}

#[test]
fn unchanged_layered_reload_preserves_failed_candidate_diagnostics() -> Result<(), Box<dyn Error>> {
    let mut state = LayeredConfigurationState::new();
    state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    state.replace(vec![
        LayeredConfigurationInput::bundled_defaults(VALID_TOML),
        LayeredConfigurationInput::user_owned(
            "[proxy]\nlisten-address = \"8.8.8.8\"\n",
            ConfigurationSourceMetadata::for_path("broken.toml"),
        ),
    ]);
    let failure_before = state.failed_candidate().cloned();

    let outcome = state.replace(vec![LayeredConfigurationInput::bundled_defaults(
        VALID_TOML,
    )]);
    assert!(matches!(
        outcome,
        LayeredConfigurationReloadOutcome::Unchanged { .. }
    ));
    assert_eq!(state.failed_candidate(), failure_before.as_ref());
    Ok(())
}

#[test]
fn oxmux_dependency_boundary_remains_headless() -> Result<(), Box<dyn Error>> {
    let manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))?;
    for forbidden in [
        "gpui", "notify", "keyring", "oauth", "reqwest", "sqlx", "rusqlite", "sled",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "oxmux should not depend on {forbidden} for layered configuration"
        );
    }
    Ok(())
}

fn assert_error_kind<T>(
    result: Result<T, CoreError>,
    kind: ConfigurationErrorKind,
    field_path: &str,
) {
    let matched = match result {
        Err(CoreError::Configuration { errors }) => {
            assert!(
                errors
                    .iter()
                    .any(|error| error.kind == kind && error.field_path == field_path),
                "expected {kind:?} at {field_path}, got {errors:?}"
            );
            true
        }
        Err(_) | Ok(_) => false,
    };

    assert!(matched, "expected configuration error");
}

fn assert_error_value<T>(
    result: Result<T, CoreError>,
    kind: ConfigurationErrorKind,
    field_path: &str,
    invalid_value: InvalidConfigurationValue,
) {
    let matched = match result {
        Err(CoreError::Configuration { errors }) => {
            assert!(
                errors.iter().any(|error| {
                    error.kind == kind
                        && error.field_path == field_path
                        && error.invalid_value == invalid_value
                }),
                "expected {kind:?} at {field_path} with {invalid_value:?}, got {errors:?}"
            );
            true
        }
        Err(_) | Ok(_) => false,
    };

    assert!(matched, "expected configuration error");
}

fn with_replace(from: &str, to: &str) -> String {
    VALID_TOML.replacen(from, to, 1)
}

fn with_streaming(streaming: &str) -> String {
    format!("{VALID_TOML}\n{streaming}")
}

fn duplicate_provider_toml() -> String {
    format!(
        "{VALID_TOML}\n[[providers]]\nid = \"mock-openai\"\nprotocol-family = \"openai\"\nrouting-eligible = true\n"
    )
}

fn duplicate_account_toml() -> String {
    with_replace(
        "credential-reference = \"mock-openai/default\"",
        "credential-reference = \"mock-openai/default\"\n\n[[providers.accounts]]\nid = \"default\"\ncredential-reference = \"mock-openai/default-copy\"",
    )
}

fn unique_temp_dir() -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(std::env::temp_dir().join(format!("oxmux-file-config-{}-{nanos}", std::process::id())))
}
