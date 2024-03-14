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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    async fn fetch_mock_exchange_rate(from: &str, to: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let mut rates = HashMap::new();
        rates.insert("USD", 1.0);
        rates.insert("EUR", 0.9);
        rates.insert("PLN", 4.0);

        let from_rate = rates.get(from).ok_or("Rate not found for source currency")?;
        let to_rate = rates.get(to).ok_or("Rate not found for target currency")?;

        Ok(to_rate / from_rate)
    }

    async fn fetch_mock_all_exchange_rates(base: &str) -> Result<Rates, Box<dyn std::error::Error>> {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.0);
        rates.insert("EUR".to_string(), 0.9);
        rates.insert("PLN".to_string(), 4.0);

        if !rates.contains_key(base) {
            return Err("Base currency not found".into());
        }

        Ok(Rates { rates })
    }

    #[tokio::test]
    async fn test_exchange_rate_conversion() {
        let from_currency = "USD";
        let to_currency = "EUR";
        let amount = 1.0;

        let rate = fetch_mock_exchange_rate(from_currency, to_currency).await.unwrap();
        let converted_amount = amount * rate;

        assert_eq!(converted_amount, 0.9);
    }

    #[tokio::test]
    async fn test_cache_logic() {
        let mut cache: HashMap<String, CacheItem> = HashMap::new();

        let from_currency = "USD";
        let to_currency = "EUR";
        let amount = 1.0;

        let rate = fetch_mock_exchange_rate(from_currency, to_currency).await.unwrap();
        cache.insert(from_currency.to_string(), CacheItem { rates: HashMap::from([(to_currency.to_string(), rate)]), timestamp: SystemTime::now() });

        let cached_rate = fetch_exchange_rate(from_currency, to_currency, &mut cache).await.unwrap();
        let converted_amount = amount * cached_rate;

        assert_eq!(converted_amount, 0.9);
    }

    #[tokio::test]
    async fn test_listing_exchange_rates() {
        let base_currency = "USD";
        let response = fetch_mock_all_exchange_rates(base_currency).await.unwrap();

        assert_eq!(response.rates.len(), 3);
        assert_eq!(response.rates.get("EUR"), Some(&0.9));
        assert_eq!(response.rates.get("PLN"), Some(&4.0));
        assert_eq!(response.rates.get("USD"), Some(&1.0));
    }
}
