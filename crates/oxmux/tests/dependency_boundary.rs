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

#[test]
fn model_registry_source_excludes_app_shell_and_runtime_imports() -> std::io::Result<()> {
    let source_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/model_registry.rs");
    let source = fs::read_to_string(source_path)?;

    for forbidden_import in [
        "use oxidemux",
        "use gpui",
        "use reqwest",
        "use hyper",
        "use ureq",
        "use tokio",
        "use smol",
        "use async_std",
        "use futures",
        "use keyring",
        "use secret_service",
    ] {
        assert!(
            !source.contains(forbidden_import),
            "model_registry source must not import {forbidden_import}"
        );
    }

    Ok(())
}
