use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{
    BoundEndpoint, ConfigurationSnapshot, CoreError, CoreHealthState, ManagementSnapshot,
    ProxyLifecycleState, QuotaSummary, UptimeMetadata, UsageSummary, core_identity,
};

pub const LOCAL_HEALTH_PATH: &str = "/health";
pub const LOCAL_HEALTH_RESPONSE_BODY: &str = "oxmux local health runtime: healthy\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalHealthRuntimeConfig {
    pub listen_address: IpAddr,
    pub port: u16,
}

impl LocalHealthRuntimeConfig {
    pub fn new(listen_address: IpAddr, port: u16) -> Result<Self, CoreError> {
        let config = Self {
            listen_address,
            port,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn loopback(port: u16) -> Self {
        Self {
            listen_address: IpAddr::from([127, 0, 0, 1]),
            port,
        }
    }

    pub fn socket_addr(self) -> SocketAddr {
        SocketAddr::new(self.listen_address, self.port)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if !self.listen_address.is_loopback() {
            return Err(CoreError::LocalRuntimeConfiguration {
                field: "listen_address",
                message: "local health runtime must bind a loopback address".to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalHealthRuntimeStatus {
    pub lifecycle: ProxyLifecycleState,
    pub health: CoreHealthState,
    pub endpoint: Option<BoundEndpoint>,
}

impl LocalHealthRuntimeStatus {
    pub fn starting() -> Self {
        Self {
            lifecycle: ProxyLifecycleState::Starting,
            health: CoreHealthState::Healthy,
            endpoint: None,
        }
    }

    pub fn failed(error: CoreError) -> Self {
        Self {
            lifecycle: ProxyLifecycleState::Failed {
                last_error: error.clone(),
            },
            health: CoreHealthState::Failed { error },
            endpoint: None,
        }
    }

    pub fn stopped(endpoint: Option<BoundEndpoint>) -> Self {
        Self {
            lifecycle: ProxyLifecycleState::Stopped,
            health: CoreHealthState::Healthy,
            endpoint,
        }
    }

    pub fn management_snapshot(&self, configuration: ConfigurationSnapshot) -> ManagementSnapshot {
        let errors = match &self.lifecycle {
            ProxyLifecycleState::Failed { last_error } => vec![last_error.clone()],
            _ => Vec::new(),
        };

        ManagementSnapshot {
            identity: core_identity(),
            lifecycle: self.lifecycle.clone(),
            health: self.health.clone(),
            configuration,
            providers: Vec::new(),
            usage: UsageSummary::zero(),
            quota: QuotaSummary::unknown(),
            warnings: Vec::new(),
            errors,
        }
    }
}

#[derive(Debug)]
pub struct LocalHealthRuntime {
    config: LocalHealthRuntimeConfig,
    endpoint: BoundEndpoint,
    started_at_unix_seconds: u64,
    started_at: Instant,
    shutdown_requested: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), CoreError>>>,
    status: LocalHealthRuntimeStatus,
}

impl LocalHealthRuntime {
    pub fn start(config: LocalHealthRuntimeConfig) -> Result<Self, CoreError> {
        config.validate()?;

        let requested_endpoint = config.socket_addr();
        let listener =
            TcpListener::bind(requested_endpoint).map_err(|error| CoreError::LocalRuntimeBind {
                endpoint: requested_endpoint.to_string(),
                message: error.to_string(),
            })?;
        listener
            .set_nonblocking(true)
            .map_err(|error| CoreError::LocalRuntimeHealthServing {
                message: format!("failed to configure listener as nonblocking: {error}"),
            })?;

        let socket_addr = listener
            .local_addr()
            .map_err(|error| CoreError::LocalRuntimeBind {
                endpoint: requested_endpoint.to_string(),
                message: error.to_string(),
            })?;
        let endpoint = BoundEndpoint { socket_addr };
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let worker_shutdown_requested = shutdown_requested.clone();
        let worker =
            thread::spawn(move || serve_health_requests(listener, worker_shutdown_requested));
        let started_at = Instant::now();
        let started_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        let status = LocalHealthRuntimeStatus {
            lifecycle: ProxyLifecycleState::Running {
                endpoint,
                uptime: UptimeMetadata {
                    started_at_unix_seconds,
                    elapsed: Duration::ZERO,
                },
            },
            health: CoreHealthState::Healthy,
            endpoint: Some(endpoint),
        };

        Ok(Self {
            config,
            endpoint,
            started_at_unix_seconds,
            started_at,
            shutdown_requested,
            worker: Some(worker),
            status,
        })
    }

    pub fn config(&self) -> LocalHealthRuntimeConfig {
        self.config
    }

    pub fn bound_endpoint(&self) -> BoundEndpoint {
        self.endpoint
    }

    pub fn status(&self) -> LocalHealthRuntimeStatus {
        match &self.worker {
            Some(worker) if worker.is_finished() => {
                LocalHealthRuntimeStatus::failed(CoreError::LocalRuntimeHealthServing {
                    message: "local health runtime worker stopped unexpectedly".to_string(),
                })
            }
            Some(_) => LocalHealthRuntimeStatus {
                lifecycle: ProxyLifecycleState::Running {
                    endpoint: self.endpoint,
                    uptime: UptimeMetadata {
                        started_at_unix_seconds: self.started_at_unix_seconds,
                        elapsed: self.started_at.elapsed(),
                    },
                },
                health: CoreHealthState::Healthy,
                endpoint: Some(self.endpoint),
            },
            None => self.status.clone(),
        }
    }

    pub fn management_snapshot(&self) -> ManagementSnapshot {
        self.status().management_snapshot(ConfigurationSnapshot {
            listen_address: self.config.listen_address,
            port: self.endpoint.socket_addr.port(),
            auto_start: false,
            logging_enabled: true,
            usage_collection_enabled: false,
            routing_default: crate::RoutingDefault::named("manual"),
            provider_references: Vec::new(),
        })
    }

    pub fn shutdown(&mut self) -> Result<LocalHealthRuntimeStatus, CoreError> {
        self.shutdown_requested.store(true, Ordering::Relaxed);

        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(Ok(())) => {
                    self.status = LocalHealthRuntimeStatus::stopped(Some(self.endpoint));
                    Ok(self.status.clone())
                }
                Ok(Err(error)) => {
                    self.status = LocalHealthRuntimeStatus::failed(error.clone());
                    Err(error)
                }
                Err(_) => {
                    let error = CoreError::LocalRuntimeShutdown {
                        message: "local health runtime worker panicked during shutdown".to_string(),
                    };
                    self.status = LocalHealthRuntimeStatus::failed(error.clone());
                    Err(error)
                }
            }
        } else {
            Ok(self.status.clone())
        }
    }
}

impl Drop for LocalHealthRuntime {
    fn drop(&mut self) {
        self.shutdown_requested.store(true, Ordering::Relaxed);

        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(Ok(())) | Ok(Err(_)) | Err(_) => {}
            }
        }
    }
}

fn serve_health_requests(
    listener: TcpListener,
    shutdown_requested: Arc<AtomicBool>,
) -> Result<(), CoreError> {
    while !shutdown_requested.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => match handle_connection(stream) {
                Ok(()) => {}
                Err(connection_error) => {
                    drop(connection_error);
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                return Err(CoreError::LocalRuntimeHealthServing {
                    message: error.to_string(),
                });
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), CoreError> {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: format!("failed to set request read timeout: {error}"),
        })?;

    let mut buffer = [0; 1024];
    let bytes_read =
        stream
            .read(&mut buffer)
            .map_err(|error| CoreError::LocalRuntimeHealthServing {
                message: error.to_string(),
            })?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_line = request.lines().next().unwrap_or_default();

    if request_line == "GET /health HTTP/1.1" || request_line == "GET /health HTTP/1.0" {
        write_response(&mut stream, "200 OK", LOCAL_HEALTH_RESPONSE_BODY)
    } else {
        write_response(&mut stream, "404 Not Found", "")
    }
}

fn write_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), CoreError> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: error.to_string(),
        })
}
