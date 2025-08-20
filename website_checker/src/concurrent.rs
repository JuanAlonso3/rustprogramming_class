// src/concurrent.rs
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::status::{CheckStatus, WebsiteStatus};
use crate::validation::Config;
use crate::time_utils::fetch_network_time_utc; // for single timestamp per batch

pub fn check_many(urls: Vec<String>, workers: usize, max_retries: usize) -> Vec<WebsiteStatus> {
    let n = urls.len();
    if n == 0 {
        return Vec::new();
    }

    let workers = workers.max(1).min(n);
    let cfg = Config::default();

    // NEW: fetch timestamp ONCE for the whole batch
    let batch_ts = Arc::new(
        fetch_network_time_utc().unwrap_or_else(|_| "unknown".to_string())
    );

    let (job_tx, job_rx) = mpsc::channel::<(usize, String)>();
    let (res_tx, res_rx) = mpsc::channel::<(usize, WebsiteStatus)>();
    let job_rx = Arc::new(Mutex::new(job_rx));

    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let rx = Arc::clone(&job_rx);
        let tx = res_tx.clone();
        let cfg = cfg.clone();
        let ts = Arc::clone(&batch_ts); // share the one timestamp

        let handle = thread::spawn(move || {
            while let Ok((idx, url)) = rx.lock().unwrap().recv() {
                // retry only on transport errors
                let mut attempts = 0usize;
                let ws = loop {
                    let ws = WebsiteStatus::request_with_timestamp(&url, &cfg, &ts);
                    match ws.status {
                        CheckStatus::Transport(_) if attempts < max_retries => {
                            attempts += 1;
                            continue; // retry
                        }
                        _ => break ws,
                    }
                };
                let _ = tx.send((idx, ws));
            }
        });
        handles.push(handle);
    }
    drop(res_tx);

    for (i, url) in urls.into_iter().enumerate() {
        let _ = job_tx.send((i, url));
    }
    drop(job_tx);

    let mut out: Vec<Option<WebsiteStatus>> = (0..n).map(|_| None).collect();
    for (idx, ws) in res_rx.iter() {
        out[idx] = Some(ws);
    }

    for h in handles {
        let _ = h.join();
    }

    out.into_iter().map(|o| o.expect("missing result")).collect()
}
