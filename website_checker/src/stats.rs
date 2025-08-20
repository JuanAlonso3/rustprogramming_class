// src/stats.rs
use crate::status::{CheckStatus, WebsiteStatus};

#[derive(Debug, Clone)]
pub struct Stats {
    pub total: usize,
    pub successes: usize,
    pub http_errors: usize,
    pub transport_errors: usize,
    pub avg_response_ms: f64,
    pub uptime_pct: f64, // successes / total * 100
}

impl Stats {
    pub fn compute(results: &[WebsiteStatus]) -> Self {
        let total = results.len();
        if total == 0 {
            return Self {
                total: 0,
                successes: 0,
                http_errors: 0,
                transport_errors: 0,
                avg_response_ms: 0.0,
                uptime_pct: 0.0,
            };
        }

        let mut successes = 0usize;
        let mut http_errors = 0usize;
        let mut transport_errors = 0usize;
        let mut total_ms: u128 = 0;

        for r in results {
            total_ms += r.response_time.as_millis();
            match r.status {
                CheckStatus::Success(_) => successes += 1,
                CheckStatus::HttpError(_) => http_errors += 1,
                CheckStatus::Transport(_) => transport_errors += 1,
            }
        }

        let avg_response_ms = (total_ms as f64) / (total as f64);
        let uptime_pct = (successes as f64) * 100.0 / (total as f64);

        Self {
            total,
            successes,
            http_errors,
            transport_errors,
            avg_response_ms,
            uptime_pct,
        }
    }

    pub fn print(&self) {
        println!("=== Summary ===");
        println!("Total: {}", self.total);
        println!("Successes: {}", self.successes);
        println!("HTTP errors: {}", self.http_errors);
        println!("Transport errors: {}", self.transport_errors);
        println!("Avg response time (ms): {:.2}", self.avg_response_ms);
        println!("Uptime: {:.2}%", self.uptime_pct);
    }
}
