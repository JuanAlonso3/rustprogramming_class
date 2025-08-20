use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;

use website_checker::concurrent;
use website_checker::stats::Stats; // stats module for computing summaries

// Reads URLs from a text file, ignoring empty lines and comments.
// Returns a vector of strings with cleaned URLs.
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
    // Load the list of websites once at startup
    let urls: Vec<String> = read_urls_from_file("src/website_list.txt")?;
    if urls.is_empty() {
        eprintln!("No URLs found in src/website_list.txt");
        return Ok(()); // exit gracefully if no URLs
    }

    // Main monitoring loop (runs indefinitely)
    loop {
        println!("=== Running website checks ===");

        // Run checks concurrently (50 threads, retry once on transport errors)
        let results = concurrent::check_many(urls.clone(), 50, 1);

        // Print individual website results
        for ws in &results {
            ws.print();
            println!("----------------------------------------");
        }

        // Compute and print summary statistics
        let summary = Stats::compute(&results);
        summary.print();

        // Wait 30 seconds before the next cycle
        println!("Sleeping 30 seconds before next run...\n");
        thread::sleep(Duration::from_secs(30));
    }
}

#[cfg(test)]
mod tests {
    use website_checker::status::{WebsiteStatus, CheckStatus};
    use std::time::Duration;

    // Test that Google returns a valid 2xx status code within 5s
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

    // Test that an invalid domain produces a transport error
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

    // Test that HTTP (non-HTTPS) URLs fail the HTTPS policy check
    #[test]
    fn http_url_violates_https_policy() {
        let ws = WebsiteStatus::request("http://example.com");
        assert!(!ws.validation.https_policy_ok);
        assert!(ws.validation.issues.iter().any(|s| s.contains("HTTPS required")));
    }
}
