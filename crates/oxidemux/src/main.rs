fn main() {
    let identity = oxmux::core_identity();

    println!("{} {}", env!("CARGO_PKG_NAME"), identity.version);
}
