// --- Production-only code (excluded during tests) ---
#[cfg(not(test))]
use serde::Deserialize;
#[cfg(not(test))]
use std::time::Duration;

#[cfg(not(test))]
const TIME_API: &str = "https://timeapi.io/api/Time/current/zone?timeZone=UTC";

#[cfg(not(test))]
#[derive(Deserialize)]
struct TimeApiResp {
    // Maps JSON field "dateTime" to this struct field
    #[serde(rename = "dateTime")]
    date_time: String,
}

#[cfg(not(test))]
pub fn fetch_network_time_utc() -> Result<String, String> {
    // If TEST_FAKE_TIME is set, return a fixed timestamp (used for integration tests)
    if std::env::var("TEST_FAKE_TIME").is_ok() {
        return Ok("2020-01-01T00:00:00Z".into());
    }

    // Make request to external time API with a 5s timeout
    match ureq::get(TIME_API).timeout(Duration::from_secs(5)).call() {
        Ok(resp) => match resp.into_json::<TimeApiResp>() {
            Ok(v) => Ok(v.date_time), // return parsed timestamp
            Err(e) => Err(format!("Failed to parse time JSON: {}", e)),
        },
        Err(e) => Err(format!("Time request failed: {}", e)),
    }
}

// --- Test-only stub (used for unit tests within this crate) ---
#[cfg(test)]
pub fn fetch_network_time_utc() -> Result<String, String> {
    // Always returns a fixed value during tests
    Ok("2020-01-01T00:00:00Z".into())
}
