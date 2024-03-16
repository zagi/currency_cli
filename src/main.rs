mod api;
mod cache;
mod config;
mod models;
use api::{fetch_all_exchange_rates, fetch_exchange_rate};
use cache::{load_cache, save_cache};
use clap::{Arg, Command};
use dotenv::dotenv;
use models::{CacheItem, Rates};
use std::{collections::HashMap, time::SystemTime};

fn main() {
    match dotenv() {
        Ok(_) => println!(".env file loaded"),
        Err(error) => println!("Warning: Failed to load .env file: {}", error),
    }

    let app = Command::new("Currency Converter")
        .version("1.0")
        .author("Michal Zagalski")
        .about("Converts currencies and lists exchange rates")
        .arg(
            Arg::new("FROM_CURRENCY")
                .help("The source currency code")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("TO_CURRENCY")
                .help("The target currency code")
                .required(false)
                .index(2),
        )
        .arg(
            Arg::new("AMOUNT")
                .help("The amount to convert")
                .required(false)
                .index(3),
        )
        .subcommand(
            Command::new("list")
                .about("Lists exchange rates for a base currency")
                .arg(
                    Arg::new("BASE_CURRENCY")
                        .help("The base currency code")
                        .default_value("PLN"),
                ),
        );

    let matches = app.get_matches();

    if let Some(("list", sub_matches)) = matches.subcommand() {
        let base_currency = sub_matches.get_one::<String>("BASE_CURRENCY").unwrap();

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
    } else {
        let from_currency = matches
            .get_one::<String>("FROM_CURRENCY")
            .expect("Source currency code is required")
            .to_uppercase();
        let to_currency = matches
            .get_one::<String>("TO_CURRENCY")
            .expect("Target currency code is required")
            .to_uppercase();
        let amount: f64 = matches
            .get_one::<String>("AMOUNT")
            .expect("Amount is required")
            .parse()
            .expect("Please type a number.");

        let mut cache = load_cache().unwrap_or_else(|_| HashMap::new());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match fetch_exchange_rate(&from_currency, &to_currency, &mut cache).await {
                Ok(rate) => {
                    let converted_amount = amount * rate;
                    println!(
                        "{} {} is {:.2} {} at an exchange rate of {:.2}",
                        amount, from_currency, converted_amount, to_currency, rate
                    );
                }
                Err(e) => eprintln!("Error fetching exchange rate: {}", e),
            }
        });
        save_cache(&cache).expect("Failed to save cache");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    async fn fetch_mock_exchange_rate(
        from: &str,
        to: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if from == "ERROR" || to == "ERROR" {
            return Err("Network error or API limit reached".into());
        }
        if from == "INVALID" || to == "INVALID" {
            return Err("Invalid currency code".into());
        }

        let rates = HashMap::from([
            ("USD".to_string(), 1.0),
            ("EUR".to_string(), 0.9),
            ("PLN".to_string(), 4.0),
        ]);

        let from_rate = rates
            .get(from)
            .ok_or("Rate not found for source currency")?;
        let to_rate = rates.get(to).ok_or("Rate not found for target currency")?;

        Ok(to_rate / from_rate)
    }

    async fn fetch_mock_all_exchange_rates(
        base: &str,
    ) -> Result<Rates, Box<dyn std::error::Error>> {
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

        let rate = fetch_mock_exchange_rate(from_currency, to_currency)
            .await
            .unwrap();
        let converted_amount = amount * rate;

        assert_eq!(converted_amount, 0.9);
    }

    #[tokio::test]
    async fn test_cache_logic() {
        let mut cache: HashMap<String, CacheItem> = HashMap::new();

        let from_currency = "USD";
        let to_currency = "EUR";
        let amount = 1.0;

        let rate = fetch_mock_exchange_rate(from_currency, to_currency)
            .await
            .unwrap();
        cache.insert(
            from_currency.to_string(),
            CacheItem {
                rates: HashMap::from([(to_currency.to_string(), rate)]),
                timestamp: SystemTime::now(),
            },
        );

        let cached_rate = fetch_exchange_rate(from_currency, to_currency, &mut cache)
            .await
            .unwrap();
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

    #[tokio::test]
    async fn test_network_error_handling() {
        let result = fetch_mock_exchange_rate("ERROR", "EUR").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_currency_code_handling() {
        let result = fetch_mock_exchange_rate("INVALID", "EUR").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_not_found_handling() {
        let result = fetch_mock_exchange_rate("USD", "INVALID").await;
        assert!(result.is_err());
    }
}
