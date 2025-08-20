use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::status::{CheckStatus, WebsiteStatus};
use crate::validation::Config;
use crate::time_utils::fetch_network_time_utc; // used to fetch a single timestamp for the batch

// Runs website checks concurrently across multiple worker threads.
// - `urls`: list of websites to check
// - `workers`: number of threads to use
// - `max_retries`: how many times to retry if a transport error occurs
// Returns a vector of WebsiteStatus results in the same order as input URLs.
pub fn check_many(urls: Vec<String>, workers: usize, max_retries: usize) -> Vec<WebsiteStatus> {
    let n = urls.len();
    if n == 0 {
        return Vec::new(); // no URLs, return empty result
    }

    // Limit workers to at least 1 and at most the number of URLs
    let workers = workers.max(1).min(n);
    let cfg = Config::default();

    // Fetch a single timestamp for the entire batch (shared across all threads)
    let batch_ts = Arc::new(
        fetch_network_time_utc().unwrap_or_else(|_| "unknown".to_string())
    );

    // Channels for sending jobs to workers and receiving results
    let (job_tx, job_rx) = mpsc::channel::<(usize, String)>();
    let (res_tx, res_rx) = mpsc::channel::<(usize, WebsiteStatus)>();
    let job_rx = Arc::new(Mutex::new(job_rx)); // wrap in Arc+Mutex so threads can share

    let mut handles = Vec::with_capacity(workers);

    // Spawn worker threads
    for _ in 0..workers {
        let rx = Arc::clone(&job_rx);
        let tx = res_tx.clone();
        let cfg = cfg.clone();
        let ts = Arc::clone(&batch_ts);

        let handle = thread::spawn(move || {
            // Process jobs until channel is closed
            while let Ok((idx, url)) = rx.lock().unwrap().recv() {
                let mut attempts = 0usize;

                // Retry loop: only retry on transport errors
                let ws = loop {
                    let ws = WebsiteStatus::request_with_timestamp(&url, &cfg, &ts);
                    match ws.status {
                        CheckStatus::Transport(_) if attempts < max_retries => {
                            attempts += 1;
                            continue; // retry on transport error
                        }
                        _ => break ws, // stop retrying on success or other error
                    }
                };

                // Send result back with original index
                let _ = tx.send((idx, ws));
            }
        });
        handles.push(handle);
    }
    drop(res_tx); // close extra result senders

    // Send jobs (URLs with their indices) to the workers
    for (i, url) in urls.into_iter().enumerate() {
        let _ = job_tx.send((i, url));
    }
    drop(job_tx); // close job sender so workers stop when done

    // Collect results into a vector, preserving input order
    let mut out: Vec<Option<WebsiteStatus>> = (0..n).map(|_| None).collect();
    for (idx, ws) in res_rx.iter() {
        out[idx] = Some(ws);
    }

    // Wait for all threads to finish
    for h in handles {
        let _ = h.join();
    }

    // Convert results from Option back to concrete WebsiteStatus
    out.into_iter().map(|o| o.expect("missing result")).collect()
}
