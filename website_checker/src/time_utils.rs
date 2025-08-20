// src/time_utils.rs

// --- Production-only bits ---
#[cfg(not(test))]
use serde::Deserialize;
#[cfg(not(test))]
use std::time::Duration;

#[cfg(not(test))]
const TIME_API: &str = "https://timeapi.io/api/Time/current/zone?timeZone=UTC";

#[cfg(not(test))]
#[derive(Deserialize)]
struct TimeApiResp {
    #[serde(rename = "dateTime")]
    date_time: String,
}

#[cfg(not(test))]
pub fn fetch_network_time_utc() -> Result<String, String> {
    // Allow integration tests to bypass external network
    if std::env::var("TEST_FAKE_TIME").is_ok() {
        return Ok("2020-01-01T00:00:00Z".into());
    }

    match ureq::get(TIME_API).timeout(Duration::from_secs(5)).call() {
        Ok(resp) => match resp.into_json::<TimeApiResp>() {
            Ok(v) => Ok(v.date_time),
            Err(e) => Err(format!("Failed to parse time JSON: {}", e)),
        },
        Err(e) => Err(format!("Time request failed: {}", e)),
    }
}

// --- Unit-test-only stub (for unit tests inside this crate) ---
#[cfg(test)]
pub fn fetch_network_time_utc() -> Result<String, String> {
    Ok("2020-01-01T00:00:00Z".into())
}
