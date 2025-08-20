use crate::time_utils::fetch_network_time_utc;
use crate::validation::{enforce_https_policy, validate_response, Config, ValidationReport};
use std::fmt;
use std::time::{Duration, Instant};
use ureq;

// Represents the result of a website check
#[derive(Debug)]
pub enum CheckStatus {
    Success(u16),       // HTTP success (2xx)
    HttpError(u16),     // Non-success HTTP status (e.g. 404, 500)
    Transport(String),  // Network/connection error (DNS, TLS, timeout, etc.)
}

// Full record of a single website check
#[derive(Debug)]
pub struct WebsiteStatus {
    pub url: String,                // website URL
    pub status: CheckStatus,        // result (success/error)
    pub response_time: Duration,    // how long the request took
    pub timestamp_utc: String,      // timestamp when check was made
    pub validation: ValidationReport, // header/body/HTTPS policy validation
}

impl WebsiteStatus {
    /// Runs a request using default validation config.
    pub fn request(url: &str) -> Self {
        Self::request_with(url, &Config::default())
    }

    /// Runs a request with a custom validation config.
    pub fn request_with(url: &str, cfg: &Config) -> Self {
        let (status, response_time, mut report) = Self::do_request(url, cfg);

        // Fetch timestamp per request (old behavior)
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

    /// Runs a request but uses a pre-fetched timestamp (avoids hitting time API repeatedly).
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

    /// Core request logic: makes the HTTP request, applies validations, but does not timestamp.
    fn do_request(url: &str, cfg: &Config) -> (CheckStatus, Duration, ValidationReport) {
        let mut report = ValidationReport::default();

        // Enforce HTTPS policy (records issues if not HTTPS)
        enforce_https_policy(url, &mut report, cfg);

        // Setup HTTP client with 5s timeout
        let start = Instant::now();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();

        // Perform request and handle results
        let (status, response_time) = match agent.get(url).call() {
            Ok(resp) => {
                let code = resp.status();
                validate_response(resp, cfg, &mut report); // run validation checks
                (CheckStatus::Success(code), start.elapsed())
            }
            Err(ureq::Error::Status(code, resp)) => {
                // Non-2xx status, but still possible to validate headers/body
                validate_response(resp, cfg, &mut report);
                (CheckStatus::HttpError(code), start.elapsed())
            }
            Err(e) => {
                // Network-level error, mark validation as failed
                report.header_ok = false;
                report.body_ok = false;
                report.issues.push(format!("Transport error: {}", e));
                (CheckStatus::Transport(e.to_string()), start.elapsed())
            }
        };

        (status, response_time, report)
    }

    /// Print the website status (uses Display implementation)
    pub fn print(&self) {
        println!("{}", self);
    }
}

// Pretty-print WebsiteStatus for console output
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
