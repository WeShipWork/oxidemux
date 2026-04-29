//! Integration tests for the oxmux dependency boundary.

use std::fs;

#[test]
fn core_manifest_excludes_app_and_ui_dependencies() -> std::io::Result<()> {
    let manifest_path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let manifest = fs::read_to_string(manifest_path)?;

    for forbidden_dependency in [
        "oxidemux",
        "gpui",
        "gpui-component",
        "tray",
        "updater",
        "keyring",
        "secret-service",
        "oauth",
        "reqwest",
        "hyper",
        "ureq",
        "tokio",
        "smol",
        "async-std",
        "futures",
        "eventsource",
        "websocket",
        "provider-sdk",
        "openai",
        "anthropic",
        "gemini",
        "packaging",
    ] {
        assert!(
            !manifest.contains(forbidden_dependency),
            "oxmux manifest must not depend on {forbidden_dependency}"
        );
    }

    Ok(())
}
