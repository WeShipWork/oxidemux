#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigurationBoundary;

use std::net::IpAddr;

use crate::CoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationSnapshot {
    pub listen_address: IpAddr,
    pub port: u16,
    pub auto_start: bool,
    pub logging_enabled: bool,
    pub usage_collection_enabled: bool,
    pub routing_default: RoutingDefault,
    pub provider_references: Vec<String>,
}

impl ConfigurationSnapshot {
    pub fn local_development() -> Self {
        Self {
            listen_address: IpAddr::from([127, 0, 0, 1]),
            port: 8787,
            auto_start: false,
            logging_enabled: true,
            usage_collection_enabled: false,
            routing_default: RoutingDefault::named("manual"),
            provider_references: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingDefault {
    pub name: String,
}

impl RoutingDefault {
    pub fn named(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    fn validate(&self) -> Result<(), CoreError> {
        if self.name.trim().is_empty() {
            return Err(CoreError::ConfigurationValidation {
                field: "routing_default",
                message: "routing default name must not be empty".to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigurationUpdateIntent {
    pub listen_address: Option<String>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub logging_enabled: Option<bool>,
    pub usage_collection_enabled: Option<bool>,
    pub routing_default: Option<RoutingDefault>,
    pub provider_references: Option<Vec<String>>,
}

impl ConfigurationUpdateIntent {
    pub fn validate(&self) -> Result<ValidatedConfigurationUpdate, CoreError> {
        let listen_address = self
            .listen_address
            .as_deref()
            .map(parse_listen_address)
            .transpose()?;

        if matches!(self.port, Some(0)) {
            return Err(CoreError::ConfigurationValidation {
                field: "port",
                message: "port must be greater than 0".to_string(),
            });
        }

        if let Some(routing_default) = &self.routing_default {
            routing_default.validate()?;
        }

        if let Some(provider_references) = &self.provider_references {
            validate_provider_references(provider_references)?;
        }

        Ok(ValidatedConfigurationUpdate {
            listen_address,
            port: self.port,
            auto_start: self.auto_start,
            logging_enabled: self.logging_enabled,
            usage_collection_enabled: self.usage_collection_enabled,
            routing_default: self.routing_default.clone(),
            provider_references: self.provider_references.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedConfigurationUpdate {
    pub listen_address: Option<IpAddr>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub logging_enabled: Option<bool>,
    pub usage_collection_enabled: Option<bool>,
    pub routing_default: Option<RoutingDefault>,
    pub provider_references: Option<Vec<String>>,
}

fn parse_listen_address(value: &str) -> Result<IpAddr, CoreError> {
    value
        .parse()
        .map_err(|_| CoreError::ConfigurationValidation {
            field: "listen_address",
            message: format!("{value:?} is not a valid IP address"),
        })
}

fn validate_provider_references(provider_references: &[String]) -> Result<(), CoreError> {
    for provider_reference in provider_references {
        if provider_reference.trim().is_empty() {
            return Err(CoreError::ConfigurationValidation {
                field: "provider_references",
                message: "provider references must not contain blank values".to_string(),
            });
        }
    }

    Ok(())
}
