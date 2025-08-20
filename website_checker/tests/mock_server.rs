// tests/mock_server.rs
//! Integration tests using a tiny mock HTTP server built with `std::net::TcpListener`.
//! No extra dependencies required.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use website_checker::status::{CheckStatus, WebsiteStatus};
use website_checker::validation::Config;

/// Start a one-shot mock server that accepts exactly one connection and replies
/// with `response`. If `delay` is Some(d), the server sleeps `d` before writing.
///
/// Returns the base URL (e.g. "http://127.0.0.1:54321") and the join handle.
fn start_mock_server(
    response: &'static str,
    delay: Option<Duration>,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let handle = thread::spawn(move || {
        // Accept exactly one connection
        if let Ok((mut stream, _peer)) = listener.accept() {
            // Read and ignore the request bytes so the client doesn't block on write.
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);

            if let Some(d) = delay {
                thread::sleep(d);
            }

            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            // stream dropped here (connection closes)
        }
    });

    (url, handle)
}

fn ok_response_html() -> &'static str {
    // Minimal valid HTTP/1.1 response with Content-Length and a small body
    "HTTP/1.1 200 OK\r\n\
     Content-Type: text/html; charset=utf-8\r\n\
     Content-Length: 11\r\n\
     X-Frame-Options: DENY\r\n\
     \r\n\
     hello world"
}

fn not_found_response() -> &'static str {
    "HTTP/1.1 404 Not Found\r\n\
     Content-Type: text/html\r\n\
     Content-Length: 9\r\n\
     \r\n\
     Not Found"
}

fn malformed_response() -> &'static str {
    // Not an HTTP response at all
    "LOL WHAT\r\n\r\n"
}

fn partial_response() -> &'static str {
    // Status line but no headers/body (invalid/unfinished)
    "HTTP/1.1 200 OK\r\n"
}

/// Helper: make a Config that disables the HTTPS policy (since mock server is http://)
fn cfg_no_https() -> Config {
    let mut cfg = Config::default();
    cfg.https_required = false;
    cfg
}

#[test]
fn mock_200_ok_and_body_validation() {
    let (url, handle) = start_mock_server(ok_response_html(), None);

    // Require body to contain the token "world" and allow text/html
    let mut cfg = cfg_no_https();
    cfg.body_contains_all = vec!["world".into()];
    cfg.content_type_allow = vec!["text/html"];

    let ws = WebsiteStatus::request_with(&url, &cfg);

    match ws.status {
        CheckStatus::Success(code) => assert_eq!(code, 200),
        other => panic!("expected success 200, got {:?}", other),
    }

    assert!(ws.validation.https_policy_ok, "HTTPS policy disabled for test");
    assert!(ws.validation.header_ok, "headers should pass");
    assert!(ws.validation.body_ok, "body should contain required token");

    handle.join().unwrap();
}

#[test]
fn mock_404_maps_to_http_error() {
    let (url, handle) = start_mock_server(not_found_response(), None);

    let ws = WebsiteStatus::request_with(&url, &cfg_no_https());

    match ws.status {
        CheckStatus::HttpError(code) => assert_eq!(code, 404),
        other => panic!("expected HttpError(404), got {:?}", other),
    }
    assert!(ws.validation.header_ok, "headers parse fine even on 404");
    assert!(ws.validation.body_ok, "no body rules means OK");

    handle.join().unwrap();
}

#[test]
fn mock_timeout_yields_transport_error() {
    // Client timeout is 5s; delay 6s to trigger it.
    let (url, handle) = start_mock_server(ok_response_html(), Some(Duration::from_secs(6)));

    let start = Instant::now();
    let ws = WebsiteStatus::request_with(&url, &cfg_no_https());
    let elapsed = start.elapsed();

    match ws.status {
        CheckStatus::Transport(_) => { /* expected */ }
        other => panic!("expected transport error due to timeout, got {:?}", other),
    }
    assert!(
        elapsed >= Duration::from_secs(5),
        "elapsed {:?} should be at least the configured timeout",
        elapsed
    );

    handle.join().unwrap();
}

#[test]
fn mock_malformed_response_is_transport_error() {
    let (url, handle) = start_mock_server(malformed_response(), None);
    let ws = WebsiteStatus::request_with(&url, &cfg_no_https());

    match ws.status {
        CheckStatus::Transport(_) => { /* expected parse failure */ }
        other => panic!("expected transport(parse) error, got {:?}", other),
    }

    handle.join().unwrap();
}

#[test]
fn mock_partial_response_is_transport_error() {
    let (url, handle) = start_mock_server(partial_response(), None);
    let ws = WebsiteStatus::request_with(&url, &cfg_no_https());

    match ws.status {
        CheckStatus::Transport(_) => { /* expected */ }
        other => panic!("expected transport error on partial response, got {:?}", other),
    }

    handle.join().unwrap();
}
