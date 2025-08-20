// src/main.rs
use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;

use website_checker::concurrent;
use website_checker::stats::Stats; // ensure lib.rs: pub mod stats;

fn read_urls_from_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|s| s.to_string())
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the list ONCE at startup.
    let urls: Vec<String> = read_urls_from_file("src/website_list.txt")?;
    if urls.is_empty() {
        eprintln!("No URLs found in src/website_list.txt");
        return Ok(());
    }

    // Periodic monitoring using the same in-memory vector.
    loop {
        println!("=== Running website checks ===");

        // 50 workers, 1 retry on transport errors
        let results = concurrent::check_many(urls.clone(), 50, 1);

        for ws in &results {
            ws.print();
            println!("----------------------------------------");
        }

        let summary = Stats::compute(&results);
        summary.print();

        println!("Sleeping 30 seconds before next run...\n");
        thread::sleep(Duration::from_secs(30));
    }
}

#[cfg(test)]
mod tests {
    use website_checker::status::{WebsiteStatus, CheckStatus};
    use std::time::Duration;

    #[test]
    fn google_returns_success() {
        let ws = WebsiteStatus::request("https://www.google.com");
        match ws.status {
            CheckStatus::Success(code) => assert!((200..=299).contains(&code)),
            other => panic!("expected success 2xx, got {:?}", other),
        }
        assert!(ws.response_time <= Duration::from_secs(5));
        assert!(!ws.timestamp_utc.is_empty() && ws.timestamp_utc != "unknown");
    }

    #[test]
    fn invalid_domain_is_transport_error() {
        let ws = WebsiteStatus::request("https://definitely-not-a-real-host.invalid");
        match ws.status {
            CheckStatus::Transport(_) => {}
            other => panic!("expected transport error, got {:?}", other),
        }
        assert!(!ws.validation.header_ok);
        assert!(!ws.validation.body_ok);
    }

    #[test]
    fn http_url_violates_https_policy() {
        let ws = WebsiteStatus::request("http://example.com");
        assert!(!ws.validation.https_policy_ok);
        assert!(ws.validation.issues.iter().any(|s| s.contains("HTTPS required")));
    }
}
