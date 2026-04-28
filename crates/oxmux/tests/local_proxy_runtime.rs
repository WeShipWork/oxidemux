use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use oxmux::{
    CoreError, CoreHealthState, LOCAL_HEALTH_RESPONSE_BODY, LocalHealthRuntime,
    LocalHealthRuntimeConfig, LocalHealthRuntimeStatus, ProxyLifecycleState,
};

#[test]
fn runtime_config_rejects_non_loopback_bind_address() {
    let error = LocalHealthRuntimeConfig::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8787)
        .expect_err("public binds must be rejected");

    assert!(matches!(
        error,
        CoreError::LocalRuntimeConfiguration {
            field: "listen_address",
            ..
        }
    ));
}

#[test]
fn runtime_binds_loopback_and_reports_actual_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    let config = LocalHealthRuntimeConfig::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)?;
    let mut runtime = LocalHealthRuntime::start(config)?;
    let endpoint = runtime.bound_endpoint();

    assert!(endpoint.socket_addr.ip().is_loopback());
    assert_ne!(endpoint.socket_addr.port(), 0);
    assert!(matches!(
        runtime.status().lifecycle,
        ProxyLifecycleState::Running { endpoint: running_endpoint, .. }
            if running_endpoint == endpoint
    ));

    runtime.shutdown()?;
    Ok(())
}

#[test]
fn health_endpoint_returns_stable_success_response() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let response = request(runtime.bound_endpoint().socket_addr, "/health")?;

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains(&format!(
        "Content-Length: {}\r\n",
        LOCAL_HEALTH_RESPONSE_BODY.len()
    )));
    assert!(response.ends_with(LOCAL_HEALTH_RESPONSE_BODY));

    runtime.shutdown()?;
    Ok(())
}

#[test]
fn health_endpoint_accepts_fragmented_request_line() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let mut stream = TcpStream::connect(runtime.bound_endpoint().socket_addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    stream.write_all(b"GET /hea")?;
    stream.write_all(b"lth HTTP/1.1\r\nHost: localhost\r\n\r\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));

    runtime.shutdown()?;
    Ok(())
}

#[test]
fn unsupported_paths_return_deterministic_non_health_response()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let response = request(runtime.bound_endpoint().socket_addr, "/missing")?;

    assert!(response.starts_with("HTTP/1.1 404 Not Found\r\n"));
    assert!(!response.contains(LOCAL_HEALTH_RESPONSE_BODY));

    runtime.shutdown()?;
    Ok(())
}

#[test]
fn bind_failure_produces_structured_failed_status() -> Result<(), Box<dyn std::error::Error>> {
    let occupied_listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))?;
    let occupied_addr = occupied_listener.local_addr()?;
    let config = LocalHealthRuntimeConfig::new(occupied_addr.ip(), occupied_addr.port())?;
    let error = LocalHealthRuntime::start(config).expect_err("occupied port must fail to bind");
    let status = LocalHealthRuntimeStatus::failed(error.clone());

    assert!(matches!(error, CoreError::LocalRuntimeBind { .. }));
    assert!(matches!(
        status.lifecycle,
        ProxyLifecycleState::Failed {
            last_error: CoreError::LocalRuntimeBind { .. }
        }
    ));
    assert!(matches!(
        status.health,
        CoreHealthState::Failed {
            error: CoreError::LocalRuntimeBind { .. }
        }
    ));

    Ok(())
}

#[test]
fn lifecycle_status_reports_starting_running_failed_and_stopped()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(matches!(
        LocalHealthRuntimeStatus::starting().lifecycle,
        ProxyLifecycleState::Starting
    ));

    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    assert!(matches!(
        runtime.status().lifecycle,
        ProxyLifecycleState::Running { .. }
    ));

    let failed = LocalHealthRuntimeStatus::failed(CoreError::LocalRuntimeBind {
        endpoint: "127.0.0.1:8787".to_string(),
        message: "address already in use".to_string(),
    });
    assert!(matches!(
        failed.lifecycle,
        ProxyLifecycleState::Failed { .. }
    ));

    let stopped = runtime.shutdown()?;
    assert!(matches!(stopped.lifecycle, ProxyLifecycleState::Stopped));
    assert!(matches!(
        runtime.status().lifecycle,
        ProxyLifecycleState::Stopped
    ));

    Ok(())
}

#[test]
fn shutdown_releases_listener_without_external_dependencies()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let socket_addr = runtime.bound_endpoint().socket_addr;
    runtime.shutdown()?;

    let rebound_listener = TcpListener::bind(socket_addr)?;
    assert_eq!(rebound_listener.local_addr()?, socket_addr);

    Ok(())
}

#[test]
fn management_snapshot_reflects_runtime_status() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let snapshot = runtime.management_snapshot();

    assert!(matches!(
        snapshot.lifecycle,
        ProxyLifecycleState::Running { .. }
    ));
    assert!(matches!(snapshot.health, CoreHealthState::Healthy));
    assert_eq!(
        snapshot.configuration.port,
        runtime.bound_endpoint().socket_addr.port()
    );

    runtime.shutdown()?;
    Ok(())
}

#[test]
fn client_io_failure_does_not_stop_health_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = LocalHealthRuntime::start(LocalHealthRuntimeConfig::loopback(0))?;
    let socket_addr = runtime.bound_endpoint().socket_addr;
    let idle_stream = TcpStream::connect(socket_addr)?;

    thread::sleep(Duration::from_millis(200));
    drop(idle_stream);

    let response = request(socket_addr, "/health")?;
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(matches!(
        runtime.status().lifecycle,
        ProxyLifecycleState::Running { .. }
    ));

    runtime.shutdown()?;
    Ok(())
}

fn request(socket_addr: SocketAddr, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(socket_addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    stream.write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}
