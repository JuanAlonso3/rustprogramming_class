# Financial Data Fetcher

Fetches and logs **Bitcoin**, **Ethereum**, and **S&P 500** prices every 10 seconds.
Each line is timestamped (UTC) and appended to its own text file.

## What it does
- Calls simple public endpoints to get live prices
- Uses a network time API for consistent UTC timestamps
- Prints results to the terminal and appends `[timestamp],price` to files

## Files written
- `bitcoin_pricing.txt`
- `ethereum_pricing.txt`
- `sp500_pricing.txt`

## Requirements
- Rust (stable)
- Internet access

`Cargo.toml` (already set):
```toml
ureq = { version = "2.6", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
