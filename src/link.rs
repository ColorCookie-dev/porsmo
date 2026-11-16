use std::ffi::OsString;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEFAULT_PORT: u16 = 7979;
const DRIFT_TOLERANCE_MS: u64 = 500;
const MAX_HEAD: usize = 8192;
const IO_TIMEOUT: Duration = Duration::from_secs(1);
const SUBSCRIBER_WRITE_TIMEOUT: Duration = Duration::from_millis(200);
const HEARTBEAT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Pomodoro,
    Timer,
    Stopwatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub kind: Kind,
    pub phase: Option<Phase>,
    pub running: bool,
    pub target_ms: u64,
    pub elapsed_ms: u64,
    pub session: u32,
    pub updated_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub host: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Status,
    Events,
    Reject(u16),
}

/// Serializes a status snapshot as the JSON served to the bar.
pub fn status_json(status: &Status) -> String {
    let kind = match status.kind {
        Kind::Pomodoro => "pomodoro",
        Kind::Timer => "timer",
        Kind::Stopwatch => "stopwatch",
    };
    let phase = status.phase.map(|phase| match phase {
        Phase::Work => "work",
        Phase::Break => "break",
        Phase::LongBreak => "long_break",
    });
    serde_json::json!({
        "kind": kind,
        "phase": phase,
        "running": status.running,
        "target_ms": status.target_ms,
        "elapsed_ms": status.elapsed_ms,
        "session": status.session,
        "updated_ms": status.updated_ms,
    })
    .to_string()
}

/// Returns true when `next` cannot be extrapolated from `prev` and must be pushed.
pub fn needs_publish(prev: Option<&Status>, next: &Status) -> bool {
    let Some(prev) = prev else {
        return true;
    };
    if prev.kind != next.kind
        || prev.phase != next.phase
        || prev.running != next.running
        || prev.target_ms != next.target_ms
        || prev.session != next.session
    {
        return true;
    }
    let expected = if prev.running {
        prev.elapsed_ms + next.updated_ms.saturating_sub(prev.updated_ms)
    } else {
        prev.elapsed_ms
    };
    expected.abs_diff(next.elapsed_ms) > DRIFT_TOLERANCE_MS
}

/// Resolves the HTTP port from a `PORSMO_PORT` value, defaulting to 7979.
pub fn port_from(value: Option<OsString>) -> u16 {
    value
        .and_then(|value| value.to_str()?.trim().parse::<u16>().ok())
        .filter(|port| *port != 0)
        .unwrap_or(DEFAULT_PORT)
}

/// Resolves the HTTP port from the environment.
pub fn port() -> u16 {
    port_from(std::env::var_os("PORSMO_PORT"))
}

/// Parses the request line and headers of an HTTP request head.
pub fn parse_request(head: &str) -> Option<Request> {
    let mut lines = head.split("\r\n");
    let mut parts = lines.next()?.split_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    if !parts.next()?.starts_with("HTTP/") {
        return None;
    }
    let mut request = Request {
        method: method.to_owned(),
        path: target.split('?').next().unwrap_or(target).to_owned(),
        host: None,
    };
    for line in lines.take_while(|line| !line.is_empty()) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if name.eq_ignore_ascii_case("host") {
            request.host = Some(value.trim().to_owned());
        }
    }
    Some(request)
}

/// Decides how the API answers `request` when listening on `port`.
pub fn route(request: &Request, port: u16) -> Route {
    let allowed = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    let host_ok = request
        .host
        .as_deref()
        .is_some_and(|host| allowed.iter().any(|allowed| allowed == host));
    if !host_ok {
        return Route::Reject(403);
    }
    let is_get = request.method == "GET";
    match request.path.as_str() {
        "/status" if is_get => Route::Status,
        "/events" if is_get => Route::Events,
        "/status" | "/events" => Route::Reject(405),
        _ => Route::Reject(404),
    }
}

struct Shared {
    status: Option<Status>,
    pushed: Option<Status>,
    subscribers: Vec<TcpStream>,
    heartbeat: Instant,
    closed: bool,
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Service Unavailable",
    }
}

fn respond(stream: &mut TcpStream, code: u16, body: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {code} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        reason(code),
        body.len()
    );
}

fn read_head(stream: &mut TcpStream) -> Option<String> {
    let mut head = Vec::new();
    let mut chunk = [0u8; 1024];
    while !head.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut chunk).ok().filter(|read| *read > 0)?;
        head.extend_from_slice(&chunk[..read]);
        if head.len() > MAX_HEAD {
            return None;
        }
    }
    Some(String::from_utf8_lossy(&head).into_owned())
}

fn sse_frame(status: &Status) -> String {
    format!("data: {}\n\n", status_json(status))
}

fn handle(mut stream: TcpStream, port: u16, shared: &Mutex<Shared>) {
    if stream.set_read_timeout(Some(IO_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(IO_TIMEOUT)).is_err()
    {
        return;
    }
    let Some(request) = read_head(&mut stream).as_deref().and_then(parse_request) else {
        respond(&mut stream, 400, r#"{"error":"bad request"}"#);
        return;
    };
    match route(&request, port) {
        Route::Status => match lock(shared).status.as_ref().map(status_json) {
            Some(body) => respond(&mut stream, 200, &body),
            None => respond(&mut stream, 503, r#"{"error":"starting"}"#),
        },
        Route::Events => {
            let mut shared = lock(shared);
            let mut opening = String::from(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: keep-alive\r\n\r\n",
            );
            if let Some(status) = &shared.status {
                opening.push_str(&sse_frame(status));
            }
            if stream.write_all(opening.as_bytes()).is_ok()
                && stream.set_write_timeout(Some(SUBSCRIBER_WRITE_TIMEOUT)).is_ok()
            {
                shared.subscribers.push(stream);
            }
        }
        Route::Reject(code) => {
            let body = serde_json::json!({ "error": reason(code) }).to_string();
            respond(&mut stream, code, &body);
        }
    }
}

fn serve(listener: TcpListener, port: u16, shared: Arc<Mutex<Shared>>) {
    for stream in listener.incoming() {
        if lock(&shared).closed {
            return;
        }
        if let Ok(stream) = stream {
            handle(stream, port, &shared);
        }
    }
}

/// Serves porsmo's state over a local HTTP API.
pub struct Link {
    shared: Arc<Mutex<Shared>>,
    port: Option<u16>,
}

impl Link {
    /// Starts the API on `port` (0 picks a free one); it stays inert when the port is taken.
    pub fn open(port: u16) -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            status: None,
            pushed: None,
            subscribers: Vec::new(),
            heartbeat: Instant::now(),
            closed: false,
        }));
        let mut link = Self {
            shared: Arc::clone(&shared),
            port: None,
        };
        let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) else {
            return link;
        };
        let Ok(actual) = listener.local_addr().map(|addr| addr.port()) else {
            return link;
        };
        thread::spawn(move || serve(listener, actual, shared));
        link.port = Some(actual);
        link
    }

    /// Returns the port the API listens on, or None when inactive.
    #[cfg(test)]
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Returns whether the API is listening.
    pub fn is_active(&self) -> bool {
        self.port.is_some()
    }

    /// Records the current status and pushes it to event subscribers when it changed.
    pub fn publish(&mut self, status: Status) {
        if !self.is_active() {
            return;
        }
        let mut shared = lock(&self.shared);
        let frame = if needs_publish(shared.pushed.as_ref(), &status) {
            shared.pushed = Some(status.clone());
            Some(sse_frame(&status))
        } else if shared.heartbeat.elapsed() >= HEARTBEAT {
            Some(String::from(": ping\n\n"))
        } else {
            None
        };
        shared.status = Some(status);
        if let Some(frame) = frame {
            shared
                .subscribers
                .retain_mut(|subscriber| subscriber.write_all(frame.as_bytes()).is_ok());
            shared.heartbeat = Instant::now();
        }
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        let Some(port) = self.port else {
            return;
        };
        {
            let mut shared = lock(&self.shared);
            shared.closed = true;
            for subscriber in shared.subscribers.drain(..) {
                let _ = subscriber.shutdown(Shutdown::Both);
            }
        }
        let _ = TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_millis(100),
        );
    }
}

/// Returns the current wall-clock time in Unix milliseconds.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};

    fn status(running: bool, elapsed_ms: u64, updated_ms: u64) -> Status {
        Status {
            kind: Kind::Pomodoro,
            phase: Some(Phase::Work),
            running,
            target_ms: 25 * 60 * 1000,
            elapsed_ms,
            session: 1,
            updated_ms,
        }
    }

    fn request(method: &str, path: &str, host: Option<&str>) -> Request {
        Request {
            method: method.to_owned(),
            path: path.to_owned(),
            host: host.map(str::to_owned),
        }
    }

    fn get(port: u16, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    fn subscribe(port: u16) -> BufReader<TcpStream> {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            stream,
            "GET /events HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"
        )
        .unwrap();
        BufReader::new(stream)
    }

    fn next_data(reader: &mut BufReader<TcpStream>) -> serde_json::Value {
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if let Some(data) = line.strip_prefix("data: ") {
                return serde_json::from_str(data.trim()).unwrap();
            }
        }
    }

    #[test]
    fn should_serialize_status_fields() {
        let value: serde_json::Value =
            serde_json::from_str(&status_json(&status(true, 1500, 42))).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "kind": "pomodoro",
                "phase": "work",
                "running": true,
                "target_ms": 1_500_000,
                "elapsed_ms": 1500,
                "session": 1,
                "updated_ms": 42,
            })
        );
    }

    #[test]
    fn should_serialize_missing_phase_as_null() {
        let mut stopwatch = status(true, 0, 0);
        stopwatch.kind = Kind::Stopwatch;
        stopwatch.phase = None;
        let value: serde_json::Value = serde_json::from_str(&status_json(&stopwatch)).unwrap();
        assert!(value["phase"].is_null());
    }

    #[test]
    fn should_publish_when_no_previous_status() {
        assert!(needs_publish(None, &status(true, 0, 0)));
    }

    #[test]
    fn should_not_publish_when_running_clock_follows_wall_clock() {
        let prev = status(true, 1000, 10_000);
        assert!(!needs_publish(Some(&prev), &status(true, 4000, 13_000)));
    }

    #[test]
    fn should_not_publish_when_paused_clock_is_unchanged() {
        let prev = status(false, 1000, 10_000);
        assert!(!needs_publish(Some(&prev), &status(false, 1000, 60_000)));
    }

    #[test]
    fn should_publish_when_running_state_changes() {
        let prev = status(true, 1000, 10_000);
        assert!(needs_publish(Some(&prev), &status(false, 1000, 10_000)));
    }

    #[test]
    fn should_publish_when_clock_is_reset() {
        let prev = status(true, 9000, 10_000);
        assert!(needs_publish(Some(&prev), &status(true, 0, 10_250)));
    }

    #[test]
    fn should_publish_when_wall_clock_jumps_ahead_of_clock() {
        let prev = status(true, 1000, 10_000);
        assert!(needs_publish(Some(&prev), &status(true, 2000, 600_000)));
    }

    #[test]
    fn should_publish_when_phase_changes() {
        let prev = status(true, 1000, 10_000);
        let mut next = status(true, 1000, 10_000);
        next.phase = Some(Phase::Break);
        assert!(needs_publish(Some(&prev), &next));
    }

    #[test]
    fn should_use_default_port_when_env_is_unset() {
        assert_eq!(port_from(None), DEFAULT_PORT);
    }

    #[test]
    fn should_use_port_from_env() {
        assert_eq!(port_from(Some("8123".into())), 8123);
    }

    #[test]
    fn should_use_default_port_when_env_is_invalid() {
        assert_eq!(port_from(Some("nope".into())), DEFAULT_PORT);
    }

    #[test]
    fn should_parse_method_and_path() {
        let parsed = parse_request("POST /toggle HTTP/1.1\r\nHost: 127.0.0.1:7979\r\n\r\n").unwrap();
        assert_eq!((parsed.method.as_str(), parsed.path.as_str()), ("POST", "/toggle"));
    }

    #[test]
    fn should_parse_host_header_case_insensitively() {
        let parsed = parse_request("GET /status HTTP/1.1\r\nhOsT:  localhost:7979 \r\n\r\n").unwrap();
        assert_eq!(parsed.host.as_deref(), Some("localhost:7979"));
    }

    #[test]
    fn should_reject_malformed_request_line() {
        assert_eq!(parse_request("garbage\r\n\r\n"), None);
    }

    #[test]
    fn should_route_get_status() {
        let req = request("GET", "/status", Some("127.0.0.1:7979"));
        assert_eq!(route(&req, 7979), Route::Status);
    }

    #[test]
    fn should_route_get_events() {
        let req = request("GET", "/events", Some("127.0.0.1:7979"));
        assert_eq!(route(&req, 7979), Route::Events);
    }

    #[test]
    fn should_accept_localhost_host() {
        let req = request("GET", "/status", Some("localhost:7979"));
        assert_eq!(route(&req, 7979), Route::Status);
    }

    #[test]
    fn should_forbid_foreign_host() {
        let req = request("GET", "/status", Some("evil.example:7979"));
        assert_eq!(route(&req, 7979), Route::Reject(403));
    }

    #[test]
    fn should_forbid_missing_host() {
        let req = request("GET", "/status", None);
        assert_eq!(route(&req, 7979), Route::Reject(403));
    }

    #[test]
    fn should_forbid_host_with_other_port() {
        let req = request("GET", "/status", Some("127.0.0.1:80"));
        assert_eq!(route(&req, 7979), Route::Reject(403));
    }

    #[test]
    fn should_reject_post_on_status_path() {
        let req = request("POST", "/status", Some("127.0.0.1:7979"));
        assert_eq!(route(&req, 7979), Route::Reject(405));
    }

    #[test]
    fn should_return_not_found_for_unknown_path() {
        let req = request("POST", "/explode", Some("127.0.0.1:7979"));
        assert_eq!(route(&req, 7979), Route::Reject(404));
    }

    #[test]
    fn should_return_unavailable_before_first_publish() {
        let link = Link::open(0);
        assert!(get(link.port().unwrap(), "/status").starts_with("HTTP/1.1 503"));
    }

    #[test]
    fn should_serve_published_status() {
        let mut link = Link::open(0);
        link.publish(status(false, 1500, 42));
        let response = get(link.port().unwrap(), "/status");
        let body = response.split("\r\n\r\n").nth(1).unwrap();
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(value["elapsed_ms"], 1500);
    }

    #[test]
    fn should_stay_inactive_when_port_is_taken() {
        let taken = TcpListener::bind("127.0.0.1:0").unwrap();
        let link = Link::open(taken.local_addr().unwrap().port());
        assert!(!link.is_active());
    }

    #[test]
    fn should_stream_current_status_on_subscribe() {
        let mut link = Link::open(0);
        link.publish(status(true, 1000, 10_000));
        let mut events = subscribe(link.port().unwrap());
        assert_eq!(next_data(&mut events)["elapsed_ms"], 1000);
    }

    #[test]
    fn should_stream_status_changes_to_subscribers() {
        let mut link = Link::open(0);
        link.publish(status(true, 1000, 10_000));
        let mut events = subscribe(link.port().unwrap());
        next_data(&mut events);
        link.publish(status(false, 1200, 10_200));
        assert_eq!(next_data(&mut events)["running"], false);
    }

    #[test]
    fn should_close_event_stream_when_link_is_dropped() {
        let mut link = Link::open(0);
        link.publish(status(true, 1000, 10_000));
        let mut events = subscribe(link.port().unwrap());
        next_data(&mut events);
        drop(link);
        let mut rest = String::new();
        let reached_eof = events.read_to_string(&mut rest).is_ok();
        assert!(reached_eof && rest.trim().is_empty());
    }
}
