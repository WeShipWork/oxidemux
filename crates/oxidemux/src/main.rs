fn main() {
    let _snapshot = oxmux::ManagementSnapshot::inert_bootstrap();

    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
