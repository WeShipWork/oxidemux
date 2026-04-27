use oxmux::core_identity;

#[test]
fn core_can_be_used_directly() {
    let identity = core_identity();

    assert_eq!(identity.crate_name, "oxmux");
    assert_eq!(identity.version, "0.1.0");
}
