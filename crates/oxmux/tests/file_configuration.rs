use std::error::Error;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oxmux::{
    AutoStartIntent, ConfigurationBoundary, ConfigurationErrorKind, ConfigurationSourceMetadata,
    CoreError, FileConfigurationState, InvalidConfigurationValue, LoggingSetting,
    ManagementSnapshot, ProtocolFamily, QuotaState, RoutingTarget, ValidatedFileConfiguration,
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
    assert!(configuration.warnings.is_empty());
    Ok(())
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
