// src/status.rs
use crate::time_utils::fetch_network_time_utc;
use crate::validation::{enforce_https_policy, validate_response, Config, ValidationReport};
use std::fmt;
use std::time::{Duration, Instant};
use ureq;

#[derive(Debug)]
pub enum CheckStatus {
    Success(u16),       // 2xx
    HttpError(u16),     // non-2xx HTTP status with server response
    Transport(String),  // timeout/DNS/TLS/etc.
}

#[derive(Debug)]
pub struct WebsiteStatus {
    pub url: String,
    pub status: CheckStatus,
    pub response_time: Duration,
    pub timestamp_utc: String,        // ISO 8601 from timeapi.io
    pub validation: ValidationReport, // header/body/https policy checks
}

impl WebsiteStatus {
    /// Simple entrypoint using default validation config (keeps old behavior).
    pub fn request(url: &str) -> Self {
        Self::request_with(url, &Config::default())
    }

    /// Allows passing a custom validation config (keeps old behavior).
    pub fn request_with(url: &str, cfg: &Config) -> Self {
        let (status, response_time, mut report) = Self::do_request(url, cfg);

        // Per-request timestamp (old behavior)
        let timestamp_utc = fetch_network_time_utc().unwrap_or_else(|e| {
            report.issues.push(format!("Timestamp fetch failed: {}", e));
            "unknown".to_string()
        });

        WebsiteStatus {
            url: url.to_string(),
            status,
            response_time,
            timestamp_utc,
            validation: report,
        }
    }

    /// NEW: Same as `request_with`, but uses a pre-fetched timestamp (no time API call here).
    pub fn request_with_timestamp(url: &str, cfg: &Config, timestamp_utc: &str) -> Self {
        let (status, response_time, report) = Self::do_request(url, cfg);
        WebsiteStatus {
            url: url.to_string(),
            status,
            response_time,
            timestamp_utc: timestamp_utc.to_string(),
            validation: report,
        }
    }

    /// Internal helper that does the HTTP request + validation (no timestamping).
    fn do_request(url: &str, cfg: &Config) -> (CheckStatus, Duration, ValidationReport) {
        let mut report = ValidationReport::default();

        // HTTPS policy first (records issue but does not short-circuit)
        enforce_https_policy(url, &mut report, cfg);

        // Fetch with 5s timeout and measure time
        let start = Instant::now();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        let (status, response_time) = match agent.get(url).call() {
            Ok(resp) => {
                let code = resp.status();
                // Validate headers/body using the response
                validate_response(resp, cfg, &mut report);
                (CheckStatus::Success(code), start.elapsed())
            }
            Err(ureq::Error::Status(code, resp)) => {
                // Non-2xx, but we can still validate headers/body from resp
                validate_response(resp, cfg, &mut report);
                (CheckStatus::HttpError(code), start.elapsed())
            }
            Err(e) => {
                // Transport error, no response to validate
                report.header_ok = false;
                report.body_ok = false;
                report.issues.push(format!("Transport error: {}", e));
                (CheckStatus::Transport(e.to_string()), start.elapsed())
            }
        };

        (status, response_time, report)
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

impl fmt::Display for WebsiteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "URL: {}", self.url)?;
        match &self.status {
            CheckStatus::Success(code) => writeln!(f, "Status: {} (success)", code)?,
            CheckStatus::HttpError(code) => writeln!(f, "Status: {} (http error)", code)?,
            CheckStatus::Transport(err) => writeln!(f, "Transport error: {}", err)?,
        }
        writeln!(f, "Response time (ms): {}", self.response_time.as_millis())?;
        writeln!(f, "Timestamp (UTC): {}", self.timestamp_utc)?;
        writeln!(f, "Validation overall ok? {}", self.validation.overall_ok())?;
        writeln!(f, " - Header ok: {}", self.validation.header_ok)?;
        writeln!(f, " - Body ok: {}", self.validation.body_ok)?;
        writeln!(f, " - HTTPS policy ok: {}", self.validation.https_policy_ok)?;
        if !self.validation.issues.is_empty() {
            writeln!(f, "Issues:")?;
            for issue in &self.validation.issues {
                writeln!(f, " * {}", issue)?;
            }
        }
        Ok(())
    }
}
