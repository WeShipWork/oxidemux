use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{
    BoundEndpoint, ConfigurationSnapshot, CoreError, CoreHealthState, ManagementSnapshot,
    MinimalProxyEngine, MinimalProxyEngineConfig, MinimalProxyRequest, MinimalProxyResponse,
    ProviderExecutor, ProxyLifecycleState, QuotaSummary, RoutingAvailabilitySnapshot,
    RoutingPolicy, UptimeMetadata, UsageSummary, core_identity,
};

pub const LOCAL_HEALTH_PATH: &str = "/health";
pub const LOCAL_HEALTH_RESPONSE_BODY: &str = "oxmux local health runtime: healthy\n";
#[cfg(test)]
const MAX_LOCAL_HEALTH_REQUEST_BYTES: usize = 8 * 1024;
const MAX_LOCAL_PROXY_REQUEST_BYTES: usize = 64 * 1024;
const LOCAL_CHAT_COMPLETIONS_PATH: &str = crate::MINIMAL_CHAT_COMPLETIONS_PATH;

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

#[derive(Clone)]
pub struct LocalProxyRouteConfig {
    pub routing_policy: RoutingPolicy,
    pub availability: RoutingAvailabilitySnapshot,
    pub provider_executor: Arc<dyn ProviderExecutor + Send + Sync>,
}

impl LocalProxyRouteConfig {
    pub fn new(
        routing_policy: RoutingPolicy,
        availability: RoutingAvailabilitySnapshot,
        provider_executor: Arc<dyn ProviderExecutor + Send + Sync>,
    ) -> Self {
        Self {
            routing_policy,
            availability,
            provider_executor,
        }
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

pub struct LocalHealthRuntime {
    config: LocalHealthRuntimeConfig,
    endpoint: BoundEndpoint,
    started_at_unix_seconds: u64,
    started_at: Instant,
    shutdown_requested: Arc<AtomicBool>,
    worker: Option<JoinHandle<Result<(), CoreError>>>,
    status: LocalHealthRuntimeStatus,
}

impl std::fmt::Debug for LocalHealthRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalHealthRuntime")
            .field("config", &self.config)
            .field("endpoint", &self.endpoint)
            .field("started_at_unix_seconds", &self.started_at_unix_seconds)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl LocalHealthRuntime {
    pub fn start(config: LocalHealthRuntimeConfig) -> Result<Self, CoreError> {
        Self::start_inner(config, None)
    }

    pub fn start_with_proxy_route(
        config: LocalHealthRuntimeConfig,
        proxy_route: LocalProxyRouteConfig,
    ) -> Result<Self, CoreError> {
        Self::start_inner(config, Some(proxy_route))
    }

    fn start_inner(
        config: LocalHealthRuntimeConfig,
        proxy_route: Option<LocalProxyRouteConfig>,
    ) -> Result<Self, CoreError> {
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
        let worker = thread::spawn(move || {
            serve_health_requests(listener, worker_shutdown_requested, proxy_route)
        });
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
    proxy_route: Option<LocalProxyRouteConfig>,
) -> Result<(), CoreError> {
    while !shutdown_requested.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => match handle_connection(stream, proxy_route.as_ref()) {
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

fn handle_connection(
    mut stream: TcpStream,
    proxy_route: Option<&LocalProxyRouteConfig>,
) -> Result<(), CoreError> {
    stream
        .set_nonblocking(false)
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: format!("failed to configure connection as blocking: {error}"),
        })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: format!("failed to set request read timeout: {error}"),
        })?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: format!("failed to set response write timeout: {error}"),
        })?;

    let request = match read_local_request(&mut stream) {
        Ok(request) => request,
        Err(error @ CoreError::MinimalProxyRequestValidation { .. }) => {
            let response = MinimalProxyResponse::invalid_request(&error);
            return write_json_response(&mut stream, response.status_code, &response.body);
        }
        Err(error) => {
            return write_connection_error_response(&mut stream, &error);
        }
    };

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", LOCAL_HEALTH_PATH) => {
            write_response(&mut stream, "200 OK", LOCAL_HEALTH_RESPONSE_BODY)
        }
        ("POST", LOCAL_CHAT_COMPLETIONS_PATH) => {
            let Some(proxy_route) = proxy_route else {
                let response = MinimalProxyResponse::unsupported_path();
                return write_json_response(&mut stream, response.status_code, &response.body);
            };
            let proxy_request = match MinimalProxyRequest::open_ai_chat_completions(request.body) {
                Ok(request) => request,
                Err(error) => {
                    let response = MinimalProxyResponse::invalid_request(&error);
                    return write_json_response(&mut stream, response.status_code, &response.body);
                }
            };
            let response = MinimalProxyEngine::execute_to_response(
                proxy_request,
                MinimalProxyEngineConfig::new(
                    &proxy_route.routing_policy,
                    &proxy_route.availability,
                    proxy_route.provider_executor.as_ref(),
                ),
            );
            write_json_response(&mut stream, response.status_code, &response.body)
        }
        _ => {
            let response = MinimalProxyResponse::unsupported_path();
            write_json_response(&mut stream, response.status_code, &response.body)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalHttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_local_request(stream: &mut TcpStream) -> Result<LocalHttpRequest, CoreError> {
    let mut request = Vec::new();
    let mut buffer = [0; 512];
    let mut header_end = None;

    while request.len() < MAX_LOCAL_PROXY_REQUEST_BYTES {
        let remaining_bytes = MAX_LOCAL_PROXY_REQUEST_BYTES - request.len();
        let read_limit = remaining_bytes.min(buffer.len());
        let bytes_read = stream.read(&mut buffer[..read_limit]).map_err(|error| {
            CoreError::LocalRuntimeHealthServing {
                message: error.to_string(),
            }
        })?;

        if bytes_read == 0 {
            break;
        }

        request.extend_from_slice(&buffer[..bytes_read]);
        if let Some(index) = find_header_end(&request) {
            header_end = Some(index);
            break;
        }
    }

    let Some(header_end) = header_end else {
        return Err(invalid_local_request(
            "headers",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request headers must terminate before the local request byte limit",
        ));
    };
    let headers = String::from_utf8_lossy(&request[..header_end]).into_owned();
    let mut header_lines = headers.split("\r\n");
    let Some(request_line) = header_lines.next() else {
        return Err(invalid_local_request(
            "request_line",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request line is required",
        ));
    };
    let mut request_line_parts = request_line.split_whitespace();
    let Some(method) = request_line_parts.next() else {
        return Err(invalid_local_request(
            "method",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request method is required",
        ));
    };
    let Some(path) = request_line_parts.next() else {
        return Err(invalid_local_request(
            "path",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request path is required",
        ));
    };
    let Some(version) = request_line_parts.next() else {
        return Err(invalid_local_request(
            "version",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request HTTP version is required",
        ));
    };
    if request_line_parts.next().is_some() || !(version == "HTTP/1.1" || version == "HTTP/1.0") {
        return Err(invalid_local_request(
            "request_line",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request line must be METHOD PATH HTTP/1.1 or METHOD PATH HTTP/1.0",
        ));
    }

    let content_length = parse_content_length(header_lines)?;
    let body_start = header_end + 4;
    let expected_len = body_start.checked_add(content_length).ok_or_else(|| {
        invalid_local_request(
            "content-length",
            crate::MinimalProxyErrorCode::RequestTooLarge,
            "request body length overflows local parser bounds",
        )
    })?;
    if expected_len > MAX_LOCAL_PROXY_REQUEST_BYTES {
        return Err(invalid_local_request(
            "body",
            crate::MinimalProxyErrorCode::RequestTooLarge,
            "request exceeds local proxy byte limit",
        ));
    }

    while request.len() < expected_len {
        let remaining_bytes = expected_len - request.len();
        let read_limit = remaining_bytes.min(buffer.len());
        let bytes_read = stream.read(&mut buffer[..read_limit]).map_err(|error| {
            CoreError::LocalRuntimeHealthServing {
                message: error.to_string(),
            }
        })?;
        if bytes_read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..bytes_read]);
    }

    if request.len() < expected_len {
        return Err(invalid_local_request(
            "body",
            crate::MinimalProxyErrorCode::UnsupportedRequestShape,
            "request body ended before declared content length",
        ));
    }

    Ok(LocalHttpRequest {
        method: method.to_string(),
        path: path.to_string(),
        body: request[body_start..expected_len].to_vec(),
    })
}

#[cfg(test)]
fn read_request_line(stream: &mut TcpStream) -> Result<String, CoreError> {
    let mut request = Vec::new();
    let mut buffer = [0; 512];

    while request.len() < MAX_LOCAL_HEALTH_REQUEST_BYTES {
        let remaining_bytes = MAX_LOCAL_HEALTH_REQUEST_BYTES - request.len();
        let read_limit = remaining_bytes.min(buffer.len());
        let bytes_read = stream.read(&mut buffer[..read_limit]).map_err(|error| {
            CoreError::LocalRuntimeHealthServing {
                message: error.to_string(),
            }
        })?;

        if bytes_read == 0 {
            break;
        }

        request.extend_from_slice(&buffer[..bytes_read]);

        if request.contains(&b'\n') {
            break;
        }
    }

    let line_end = request
        .iter()
        .position(|byte| *byte == b'\n')
        .unwrap_or(request.len());
    let mut request_line = &request[..line_end];

    if request_line.ends_with(b"\r") {
        request_line = &request_line[..request_line.len() - 1];
    }

    Ok(String::from_utf8_lossy(request_line).into_owned())
}

fn find_header_end(request: &[u8]) -> Option<usize> {
    request.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length<'a>(
    header_lines: impl Iterator<Item = &'a str>,
) -> Result<usize, CoreError> {
    let mut content_length = None;
    for line in header_lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            let parsed_content_length = value.trim().parse::<usize>().map_err(|_| {
                invalid_local_request(
                    "content-length",
                    crate::MinimalProxyErrorCode::UnsupportedRequestShape,
                    "content-length must be a non-negative integer",
                )
            })?;

            if matches!(content_length, Some(existing) if existing != parsed_content_length) {
                return Err(invalid_local_request(
                    "content-length",
                    crate::MinimalProxyErrorCode::UnsupportedRequestShape,
                    "duplicate content-length headers must agree",
                ));
            }

            content_length = Some(parsed_content_length);
        }
    }

    Ok(content_length.unwrap_or(0))
}

fn invalid_local_request(
    field: &'static str,
    code: crate::MinimalProxyErrorCode,
    message: impl Into<String>,
) -> CoreError {
    CoreError::MinimalProxyRequestValidation {
        field,
        code,
        message: message.into(),
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

fn write_connection_error_response(
    stream: &mut TcpStream,
    error: &CoreError,
) -> Result<(), CoreError> {
    let status_code = if matches!(error, CoreError::LocalRuntimeHealthServing { .. }) {
        408
    } else {
        500
    };
    let body = serde_json::json!({
        "error": {
            "code": "local_runtime_io",
            "message": error.to_string(),
            "type": "oxmux_proxy_error"
        }
    })
    .to_string();

    write_json_response(stream, status_code, &body)
}

fn write_json_response(
    stream: &mut TcpStream,
    status_code: u16,
    body: &str,
) -> Result<(), CoreError> {
    let reason = match status_code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        408 => "Request Timeout",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        _ => "Internal Server Error",
    };
    let response = format!(
        "HTTP/1.1 {status_code} {reason}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| CoreError::LocalRuntimeHealthServing {
            message: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::thread;

    use super::{CoreError, MAX_LOCAL_HEALTH_REQUEST_BYTES, read_request_line};

    #[test]
    fn request_line_reader_enforces_byte_cap() -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let reader = thread::spawn(move || {
            let (mut stream, _) =
                listener
                    .accept()
                    .map_err(|error| CoreError::LocalRuntimeHealthServing {
                        message: error.to_string(),
                    })?;
            read_request_line(&mut stream)
        });

        let mut stream = TcpStream::connect(socket_addr)?;
        match stream.write_all(&vec![b'a'; MAX_LOCAL_HEALTH_REQUEST_BYTES + 512]) {
            Ok(()) => match stream.shutdown(Shutdown::Write) {
                Ok(()) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        ErrorKind::BrokenPipe
                            | ErrorKind::ConnectionReset
                            | ErrorKind::NotConnected
                    ) => {}
                Err(error) => return Err(error.into()),
            },
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::BrokenPipe | ErrorKind::ConnectionReset | ErrorKind::NotConnected
                ) => {}
            Err(error) => return Err(error.into()),
        }

        let request_line = match reader.join() {
            Ok(result) => result?,
            Err(_) => return Err("request reader thread panicked".into()),
        };

        assert_eq!(request_line.len(), MAX_LOCAL_HEALTH_REQUEST_BYTES);
        Ok(())
    }
}
