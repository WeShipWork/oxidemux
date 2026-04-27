pub mod configuration;
pub mod errors;
pub mod management;
pub mod protocol;
pub mod provider;
pub mod routing;
pub mod streaming;
pub mod usage;

pub use errors::CoreError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreIdentity {
    pub crate_name: &'static str,
    pub version: &'static str,
}

pub const CORE_IDENTITY: CoreIdentity = CoreIdentity {
    crate_name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

pub fn core_identity() -> CoreIdentity {
    CORE_IDENTITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_identity() {
        assert_eq!(core_identity().crate_name, "oxmux");
        assert_eq!(core_identity().version, "0.1.0");
    }
}
