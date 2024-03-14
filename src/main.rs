use reqwest;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, env, time::{Duration, SystemTime}};
use dotenv::dotenv;

#[derive(Serialize, Deserialize)]
struct Rates {
    rates: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheItem {
    rates: HashMap<String, f64>,
    timestamp: SystemTime,
}

static CACHE_DURATION: Duration = Duration::new(3600, 0); // 1 hour

async fn fetch_exchange_rate(from: &str, to: &str, cache: &mut HashMap<String, CacheItem>) -> Result<f64, Box<dyn std::error::Error>> {
    if let Some(cached_item) = cache.get(from) {
        if SystemTime::now().duration_since(cached_item.timestamp)?.as_secs() < CACHE_DURATION.as_secs() {
            if let Some(rate) = cached_item.rates.get(to) {
                return Ok(*rate);
            }
        }
    }

    let api_key = env::var("API_KEY")?;
    let api_url = format!("https://api.exchangerate-api.com/v4/latest/{}?access_key={}", from, api_key);
    let response: Rates = reqwest::get(api_url).await?.json().await?;
    cache.insert(from.to_string(), CacheItem { rates: response.rates.clone(), timestamp: SystemTime::now() });

    response.rates.get(to).ok_or_else(|| "Rate not found in response".into()).copied()
}

async fn fetch_all_exchange_rates(base: &str) -> Result<Rates, Box<dyn std::error::Error>> {
    let api_key = env::var("API_KEY")?;
    let api_url = format!("https://api.exchangerate-api.com/v4/latest/{}?access_key={}", base, api_key);
    let response: Rates = reqwest::get(api_url).await?.json().await?;
    Ok(response)
}

fn main() {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && args[1] == "list" {
        let base_currency = if args.len() == 3 { &args[2] } else { "PLN" };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match fetch_all_exchange_rates(base_currency).await {
                Ok(api_response) => {
                    println!("Exchange rates for {}:", base_currency);
                    for (currency, rate) in api_response.rates.iter() {
                        println!("{}: {}", currency, rate);
                    }
                }
                Err(e) => eprintln!("Error fetching exchange rates: {}", e),
            }
        });
    } else if args.len() == 4 {
        let from_currency = &args[1];
        let to_currency = &args[2];
        let amount: f64 = args[3].parse().expect("Invalid amount");

        let mut cache: HashMap<String, CacheItem> = HashMap::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match fetch_exchange_rate(from_currency, to_currency, &mut cache).await {
                Ok(rate) => {
                    let converted_amount = amount * rate;
                    println!("{} {} is {} {}", amount, from_currency, converted_amount, to_currency);
                }
                Err(e) => eprintln!("Error fetching exchange rate: {}", e),
            }
        });
    } else {
        println!("Usage: currency_converter <from_currency> <to_currency> <amount>");
        println!("Or: currency_converter list [base_currency]");
    }
}
