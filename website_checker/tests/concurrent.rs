use website_checker::concurrent::check_many;
use website_checker::status::{CheckStatus, WebsiteStatus};

/// Helper: run sequentially using the same API for comparison.
fn check_sequential(urls: &[String]) -> Vec<WebsiteStatus> {
    urls.iter().map(|u| WebsiteStatus::request(u)).collect()
}

#[test]
fn concurrent_matches_sequential_for_basic_cases() {
    // Keep this small and stable to avoid flakiness.
    let urls = vec![
        "https://www.google.com".to_string(),
        "https://definitely-not-a-real-host.invalid".to_string(),
    ];

    let conc = check_many(urls.clone(), /*workers=*/2, /*max_retries=*/1);
    let seq  = check_sequential(&urls);

    assert_eq!(conc.len(), seq.len());

    for (c, s) in conc.iter().zip(seq.iter()) {
        assert_eq!(c.url, s.url, "URL order or association changed");
        match (&c.status, &s.status) {
            (CheckStatus::Success(cc), CheckStatus::Success(sc)) => {
                // Both succeeded; codes should be in 2xx
                assert!((200..=299).contains(cc));
                assert!((200..=299).contains(sc));
            }
            (CheckStatus::HttpError(cc), CheckStatus::HttpError(sc)) => {
                // Both HTTP errors; just ensure they are non-2xx
                assert!(! (200..=299).contains(cc));
                assert!(! (200..=299).contains(sc));
            }
            (CheckStatus::Transport(_), CheckStatus::Transport(_)) => { /* ok */ }
            (a, b) => panic!("Status kinds differ: concurrent={:?}, sequential={:?}", a, b),
        }
    }
}

#[test]
fn concurrent_preserves_input_order() {
    let urls = vec![
        "https://www.google.com".to_string(),
        "https://definitely-not-a-real-host.invalid".to_string(),
    ];

    let conc = check_many(urls.clone(), /*workers=*/2, /*max_retries=*/0);

    // Results should correspond to input indices.
    assert_eq!(conc[0].url, urls[0]);
    assert_eq!(conc[1].url, urls[1]);
}
