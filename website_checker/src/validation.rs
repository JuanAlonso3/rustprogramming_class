use std::io::Read;
use ureq;

// Holds results of validation checks on headers, body, and HTTPS policy
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub header_ok: bool,
    pub body_ok: bool,
    pub https_policy_ok: bool,
    pub issues: Vec<String>, // detailed issues found
}

impl ValidationReport {
    // Overall pass/fail: true only if all categories pass
    pub fn overall_ok(&self) -> bool {
        self.header_ok && self.body_ok && self.https_policy_ok
    }
}

// Validation configuration options (rules to enforce)
#[derive(Clone)]
pub struct Config {
    // HTTPS policy
    pub https_required: bool,

    // Header validation rules
    pub required_headers: Vec<&'static str>,         // must exist
    pub content_type_allow: Vec<&'static str>,       // allowlist
    pub header_equals: Vec<(&'static str, String)>,  // exact matches
    pub header_contains: Vec<(&'static str, String)>,// substring matches

    // Body validation rules
    pub max_body_bytes: usize,       // max body size to read
    pub body_contains_all: Vec<String>, // must contain all
    pub body_contains_any: Vec<String>, // must contain at least one
}

// Default validation configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            https_required: true,
            required_headers: vec!["Content-Type"],
            content_type_allow: vec!["text/html", "application/json"],
            header_equals: vec![],
            header_contains: vec![],
            max_body_bytes: 64 * 1024, // 64 KB
            body_contains_all: vec![],
            body_contains_any: vec![],
        }
    }
}

/// Enforce HTTPS-only policy (records an issue if violated).
pub fn enforce_https_policy(url: &str, report: &mut ValidationReport, cfg: &Config) {
    if !cfg.https_required {
        report.https_policy_ok = true;
        return;
    }
    if url.starts_with("https://") {
        report.https_policy_ok = true;
    } else {
        report.https_policy_ok = false;
        report.issues.push("HTTPS required by policy, but URL is not https".into());
    }
}

/// Validate response headers and (optionally) body.
pub fn validate_response(resp: ureq::Response, cfg: &Config, report: &mut ValidationReport) {
    // First check headers
    validate_headers(&resp, cfg, report);

    // Check body only if rules are configured
    let need_body = !cfg.body_contains_all.is_empty() || !cfg.body_contains_any.is_empty();
    if need_body {
        validate_body(resp, cfg, report);
    } else {
        report.body_ok = true;
    }
}

// Header validation helper
fn validate_headers(resp: &ureq::Response, cfg: &Config, report: &mut ValidationReport) {
    let mut ok = true;

    // Check required headers exist
    for &h in &cfg.required_headers {
        if resp.header(h).is_none() {
            ok = false;
            report.issues.push(format!("Missing header: {}", h));
        }
    }

    // Check Content-Type allowlist
    if !cfg.content_type_allow.is_empty() {
        match resp.header("Content-Type") {
            Some(ct) => {
                let lower = ct.to_ascii_lowercase();
                if !cfg.content_type_allow.iter()
                    .any(|allowed| lower.starts_with(&allowed.to_ascii_lowercase()))
                {
                    ok = false;
                    report.issues.push(format!("Content-Type not allowed: {}", ct));
                }
            }
            None => {
                ok = false;
                report.issues.push("Missing header: Content-Type".into());
            }
        }
    }

    // Exact header matches
    for (name, expected) in &cfg.header_equals {
        match resp.header(name) {
            Some(v) if v == expected => {}
            Some(v) => {
                ok = false;
                report.issues.push(format!(
                    "Header {} mismatch: got '{}', expected '{}'",
                    name, v, expected
                ));
            }
            None => {
                ok = false;
                report.issues.push(format!("Missing header: {}", name));
            }
        }
    }

    // Header substring matches
    for (name, needle) in &cfg.header_contains {
        match resp.header(name) {
            Some(v) if v.contains(needle) => {}
            Some(v) => {
                ok = false;
                report.issues.push(format!(
                    "Header {} does not contain '{}': got '{}'",
                    name, needle, v
                ));
            }
            None => {
                ok = false;
                report.issues.push(format!("Missing header: {}", name));
            }
        }
    }

    report.header_ok = ok;
}

/// Check if `needle` appears in text as a standalone word.
/// Falls back to substring if `needle` has non-alphanumeric chars.
fn contains_token(text: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let is_wordy = needle.chars().all(|c| c.is_alphanumeric());
    if !is_wordy {
        return text.contains(needle);
    }

    let bytes = text.as_bytes();
    let nbytes = needle.as_bytes();
    let nlen = nbytes.len();
    if nlen > bytes.len() {
        return false;
    }

    let mut i = 0;
    while let Some(pos) = text[i..].find(needle) {
        let start = i + pos;
        let end = start + nlen;

        // Ensure word boundaries
        let left_ok = if start == 0 { true } else { !bytes[start - 1].is_ascii_alphanumeric() };
        let right_ok = if end >= bytes.len() { true } else { !bytes[end].is_ascii_alphanumeric() };

        if left_ok && right_ok {
            return true;
        }

        i = start + 1; // continue search
    }
    false
}

/// Validate body text according to config (ALL-of and ANY-of rules).
pub fn check_body_text(text: &str, cfg: &Config) -> (bool, Vec<String>) {
    let mut issues = Vec::new();

    // ALL-of rules
    for needle in &cfg.body_contains_all {
        if !contains_token(text, needle) {
            issues.push(format!("Body missing required text: '{}'", needle));
        }
    }

    // ANY-of rules
    let mut ok = issues.is_empty();
    if !cfg.body_contains_any.is_empty() {
        let any_hit = cfg.body_contains_any.iter().any(|n| contains_token(text, n));
        if !any_hit {
            issues.push(format!("Body did not contain ANY of: {:?}", cfg.body_contains_any));
        }
        ok = ok && any_hit;
    }

    (ok, issues)
}

// Body validation helper: reads body and applies text checks
fn validate_body(resp: ureq::Response, cfg: &Config, report: &mut ValidationReport) {
    let mut reader = resp.into_reader().take(cfg.max_body_bytes as u64);
    let mut buf = Vec::new();
    if let Err(e) = reader.read_to_end(&mut buf) {
        report.body_ok = false;
        report.issues.push(format!("Failed to read response body: {}", e));
        return;
    }

    let text = String::from_utf8_lossy(&buf);
    let (ok, issues) = check_body_text(&text, cfg);
    report.body_ok = ok;
    report.issues.extend(issues);
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_policy_allows_https_and_blocks_http() {
        let cfg = Config::default();

        let mut rep_https = ValidationReport::default();
        enforce_https_policy("https://example.com", &mut rep_https, &cfg);
        assert!(rep_https.https_policy_ok);
        assert!(rep_https.issues.is_empty());

        let mut rep_http = ValidationReport::default();
        enforce_https_policy("http://example.com", &mut rep_http, &cfg);
        assert!(!rep_http.https_policy_ok);
        assert!(rep_http.issues.iter().any(|s| s.contains("HTTPS required")));
    }

    #[test]
    fn body_text_all_and_any_modes() {
        let mut cfg = Config::default();
        cfg.body_contains_all = vec!["Welcome".into(), "Home".into()];
        cfg.body_contains_any = vec!["Login".into(), "Sign".into()];

        // Has all "ALL-of" and one "ANY-of"
        let (ok1, issues1) = check_body_text("Welcome to my Home page. Please Login.", &cfg);
        assert!(ok1);
        assert!(issues1.is_empty());

        // Missing "Home" and fails ANY-of
        let (ok2, issues2) = check_body_text("Welcome area only.", &cfg);
        assert!(!ok2);
        assert!(issues2.iter().any(|s| s.contains("Body missing required text: 'Home'")));
        assert!(issues2.iter().any(|s| s.contains("Body did not contain ANY of")));

        // Only ANY-of configured
        let mut cfg2 = Config::default();
        cfg2.body_contains_any = vec!["one".into(), "two".into()];
        let (ok3, issues3) = check_body_text("zero and two present", &cfg2);
        assert!(ok3);
        assert!(issues3.is_empty());

        let (ok4, issues4) = check_body_text("none present", &cfg2);
        assert!(!ok4);
        assert!(issues4.iter().any(|s| s.contains("ANY of")));
    }
}
