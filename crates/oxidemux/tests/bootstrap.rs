//! Integration tests for the oxidemux app-shell bootstrap output.

use std::process::Command;

#[test]
fn package_metadata_matches_manifest() {
    assert_eq!(env!("CARGO_PKG_NAME"), "oxidemux");
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
}

#[test]
fn binary_preserves_metadata_output() -> std::io::Result<()> {
    let binary_path = env!("CARGO_BIN_EXE_oxidemux");
    let output = Command::new(binary_path).output()?;

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "oxidemux 0.1.0\n");

    Ok(())
}

#[test]
fn app_shell_status_contract_comes_from_oxmux_facade() {
    let snapshot = oxmux::ManagementSnapshot::inert_bootstrap();

    assert_eq!(snapshot.identity.crate_name, "oxmux");
    assert!(!snapshot.identity.version.is_empty());
    assert!(matches!(
        snapshot.lifecycle,
        oxmux::ProxyLifecycleState::Stopped
    ));
    assert!(matches!(snapshot.health, oxmux::CoreHealthState::Healthy));
}

#[test]
fn app_shell_exercises_health_runtime_through_oxmux() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime =
        oxmux::LocalHealthRuntime::start(oxmux::LocalHealthRuntimeConfig::loopback(0))?;
    let snapshot = runtime.management_snapshot();

    assert!(matches!(
        snapshot.lifecycle,
        oxmux::ProxyLifecycleState::Running { .. }
    ));
    assert!(snapshot.configuration.listen_address.is_loopback());

    runtime.shutdown()?;
    Ok(())
}
