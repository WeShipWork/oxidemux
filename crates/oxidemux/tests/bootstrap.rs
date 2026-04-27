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
