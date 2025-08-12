
// Crates used: ureq (HTTP), serde (typed JSON), std (time, file I/O)
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

// ========================= Shared (trait, time, IO) =========================

// Network time (UTC)
const TIME_API: &str = "https://timeapi.io/api/Time/current/zone?timeZone=UTC";

//There is only 3 we cases we care about when working with http api calls
#[derive(Debug)]
pub enum ApiResult {
    Success(f64),
    ApiError(String),
    NetworkError(String),
}

// Declaring the Shared pricing trait
pub trait Pricing {
    fn fetch_price(&self) -> ApiResult;
    fn save_to_file(&self, timestamp: &str, price: f64) -> std::io::Result<()>;
    fn display_name(&self) -> &'static str;
}

// Typed model for timeapi.io
#[derive(Deserialize)]
struct TimeApiResp {
    #[serde(rename = "dateTime")]
    date_time: String,
}

//Handles the time/date api request
fn fetch_network_time_utc() -> Result<String, String> {
    match ureq::get(TIME_API).call() {
        Ok(resp) => match resp.into_json::<TimeApiResp>() {
            Ok(v) => Ok(v.date_time),
            Err(e) => Err(format!("Failed to parse time JSON: {}", e)),
        },
        Err(e) => Err(format!("Time request failed: {}", e)),
    }
}


//Just writes the asset price/timestamp to its respective asset txt file
fn write_price_to_file(file_name: &str, timestamp: &str, price: f64) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;
    writeln!(file, "[{}],{}", timestamp, price)?;
    Ok(())
}

// ============================== Bitcoin (Binance US) ==============================

//declaring Api link and file name
const BITCOIN_API: &str = "https://api.binance.us/api/v3/ticker/price?symbol=BTCUSD";
const BITCOIN_FILE_NAME: &str = "bitcoin_pricing.txt";

struct Bitcoin;

#[derive(Deserialize)]
struct BinancePrice {
    price: String, // Binance returns the price as a string
}

//This request the price from the API url
impl Pricing for Bitcoin {
    fn fetch_price(&self) -> ApiResult {
        match ureq::get(BITCOIN_API).call() {
            Ok(response) => {
                if response.status() == 200 {
                    match response.into_json::<BinancePrice>() {
                        Ok(v) => match v.price.parse::<f64>() {
                            Ok(p) => ApiResult::Success(p),
                            Err(e) => ApiResult::ApiError(format!("Failed to parse price: {}", e)),
                        },
                        Err(e) => ApiResult::ApiError(format!("Failed to parse JSON: {}", e)),
                    }
                } else {
                    ApiResult::ApiError(format!("HTTP error: {}", response.status()))
                }
            }
            Err(e) => ApiResult::NetworkError(format!("Request failed: {}", e)),
        }
    }

    //Just saves the date/price to a txt file
    fn save_to_file(&self, timestamp: &str, price: f64) -> std::io::Result<()> {
        write_price_to_file(BITCOIN_FILE_NAME, timestamp, price)
    }

    //Returns the name of the asset
    fn display_name(&self) -> &'static str {
        "Bitcoin"
    }
}

// ============================== Ethereum (Binance US) ==============================

//declaring Api link, file name, and struct
const ETHEREUM_API: &str = "https://api.binance.us/api/v3/ticker/price?symbol=ETHUSD";
const ETHEREUM_FILE_NAME: &str = "ethereum_pricing.txt";
struct Ethereum;

//This request the price from the API urls
impl Pricing for Ethereum {
    fn fetch_price(&self) -> ApiResult {
        match ureq::get(ETHEREUM_API).call() {
            Ok(response) => {
                if response.status() == 200 {
                    match response.into_json::<BinancePrice>() {
                        Ok(v) => match v.price.parse::<f64>() {
                            Ok(p) => ApiResult::Success(p),
                            Err(e) => ApiResult::ApiError(format!("Failed to parse price: {}", e)),
                        },
                        Err(e) => ApiResult::ApiError(format!("Failed to parse JSON: {}", e)),
                    }
                } else {
                    ApiResult::ApiError(format!("HTTP error: {}", response.status()))
                }
            }
            Err(e) => ApiResult::NetworkError(format!("Request failed: {}", e)),
        }
    }
    //Just saves the date/price to a txt file
    fn save_to_file(&self, timestamp: &str, price: f64) -> std::io::Result<()> {
        write_price_to_file(ETHEREUM_FILE_NAME, timestamp, price)
    }
    //returns the name of the asset
    fn display_name(&self) -> &'static str {
        "Ethereum"
    }
}

// ============================== S&P 500 (Stooq) ==============================

//declaring Api link, file name, and struct 
const SP500_API: &str = "https://stooq.pl/q/l/?s=%5Espx&f=sd2t2ohlcv&h&e=json";
const SP500_FILE_NAME: &str = "sp500_pricing.txt";
struct Sp500;

#[derive(Deserialize)]
struct StooqResponse {
    symbols: Vec<StooqSymbol>,
}

#[derive(Deserialize)]
struct StooqSymbol {
    #[serde(deserialize_with = "de_str_or_f64")]
    close: f64,
}

// Custom deserializer to accept either a number or a string for "close"
fn de_str_or_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = f64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a float or a string containing a float")
        }
        fn visit_f64<E>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }
        fn visit_u64<E>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_i64<E>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_str<E>(self, v: &str) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            v.parse::<f64>().map_err(|_| E::custom("invalid float string"))
        }
        fn visit_string<E>(self, v: String) -> Result<f64, E>
        where
            E: serde::de::Error,
        {
            v.parse::<f64>().map_err(|_| E::custom("invalid float string"))
        }
    }
    deserializer.deserialize_any(Visitor)
}


//This request the price from the API urls
impl Pricing for Sp500 {
    fn fetch_price(&self) -> ApiResult {
        match ureq::get(SP500_API).call() {
            Ok(response) => {
                if response.status() == 200 {
                    match response.into_json::<StooqResponse>() {
                        Ok(v) => {
                            if let Some(first) = v.symbols.get(0) {
                                ApiResult::Success(first.close)
                            } else {
                                ApiResult::ApiError("No symbols in Stooq response".to_string())
                            }
                        }
                        Err(e) => ApiResult::ApiError(format!("Failed to parse JSON: {}", e)),
                    }
                } else {
                    ApiResult::ApiError(format!("HTTP error: {}", response.status()))
                }
            }
            Err(e) => ApiResult::NetworkError(format!("Request failed: {}", e)),
        }
    }
    //Just saves the date/price to a txt file
    fn save_to_file(&self, timestamp: &str, price: f64) -> std::io::Result<()> {
        write_price_to_file(SP500_FILE_NAME, timestamp, price)
    }
    //returns the name of the asset
    fn display_name(&self) -> &'static str {
        "S&P 500"
    }
}

// ================================== main ==================================

fn main() {
    // Make a list of the three things we track; each knows how to get its price and save it
    let assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin),
        Box::new(Ethereum),
        Box::new(Sp500),
    ];

    loop {
        
        let timestamp = match fetch_network_time_utc() {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("Time fetch error: {}", e);
                "unknown-time".to_string()
            }
        };

        // Go through each asset: get its latest number, show it, and save it
        for asset in &assets {
            match asset.fetch_price() {
                // Got a real price: print it and try to write a line to that asset's file
                ApiResult::Success(price) => {
                    println!("[{}] {} price: ${}", timestamp, asset.display_name(), price);
                    if let Err(e) = asset.save_to_file(&timestamp, price) {
                        eprintln!("Failed to write {} price: {}", asset.display_name(), e);
                    }
                }
                // The website answered, but the data wasn't usable
                ApiResult::ApiError(err) => {
                    eprintln!("[{}] {} API error: {}", timestamp, asset.display_name(), err);
                }
                // We couldn't reach the website at all (network issue)
                ApiResult::NetworkError(err) => {
                    eprintln!("[{}] {} Network error: {}", timestamp, asset.display_name(), err);
                }
            }
        }

        // Wait 10 seconds
        thread::sleep(Duration::from_secs(10));
    }
}
