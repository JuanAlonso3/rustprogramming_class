# Website Checker

A Rust-based tool for monitoring website availability, response times, and basic content validation.  
It runs multiple checks concurrently, validates responses, and prints detailed results and summaries.

---

## Features
- **Concurrent Website Checks**: Runs many checks in parallel using worker threads.
- **Status Tracking**: Records HTTP success (2xx), HTTP errors (non-2xx), and transport errors (DNS, TLS, timeouts).
- **Response Validation**:
  - Enforces HTTPS-only policy
  - Checks required headers and allowed content types
  - Optional body content validation
- **Statistics Reporting**: Prints total checks, success/error counts, average response time, and uptime percentage.
- **Timestamps**: Associates each batch of checks with a UTC timestamp (fetched via [timeapi.io](https://timeapi.io)).

---

## Project Structure
- `src/main.rs` – Entry point; loads URLs, runs checks in a loop, prints results and stats.
- `src/status.rs` – Core logic for making requests and validating responses.
- `src/concurrent.rs` – Runs website checks concurrently across worker threads.
- `src/stats.rs` – Computes and prints summary statistics.
- `src/validation.rs` – Rules for validating HTTPS, headers, and response body.
- `src/time_utils.rs` – Fetches network-based UTC timestamps (stubbed in tests).
- `src/website_list.txt` – List of URLs to monitor (one per line, `#` for comments).

---

## Usage
1. Add websites to `src/website_list.txt` (one URL per line).
2. Build and run the program:

```bash
cargo run
