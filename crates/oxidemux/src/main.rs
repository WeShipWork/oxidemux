//! Desktop app-shell entrypoint for OxideMux platform integration.
//!
//! `oxidemux` consumes the `oxmux` headless core facade and is responsible for
//! app-shell concerns such as future GPUI presentation, desktop lifecycle,
//! notifications, packaging, updater UX, and OS-specific integration. Core proxy
//! semantics for routing, provider/account state, protocol compatibility,
//! usage/quota state, management snapshots, auth/session state, and request
//! rewriting remain owned by `oxmux`.

#![warn(missing_docs)]

fn main() {
    let _snapshot = oxmux::ManagementSnapshot::inert_bootstrap();

    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
