use crate::models::{CacheItem, Rates};
use reqwest::StatusCode;
use std::{collections::HashMap, env, error::Error, time::SystemTime};

use crate::config::CACHE_DURATION;

pub async fn fetch_exchange_rate(
    from: &str,
    to: &str,
    cache: &mut HashMap<String, CacheItem>,
) -> Result<f64, Box<dyn Error>> {
    if let Some(cached_item) = cache.get(from) {
        if SystemTime::now()
            .duration_since(cached_item.timestamp)?
            .as_secs()
            < CACHE_DURATION.as_secs()
        {
            if let Some(rate) = cached_item.rates.get(to) {
                return Ok(*rate);
            }
        }
    }

    let api_key = env::var("API_KEY")?;
    let api_url = format!(
        "https://api.exchangerate-api.com/v4/latest/{}?access_key={}",
        from, api_key
    );

    let response = reqwest::get(&api_url).await?;

    match response.status() {
        StatusCode::OK => {
            let rates: Rates = response.json().await?;
            cache.insert(
                from.to_string(),
                CacheItem {
                    rates: rates.rates.clone(),
                    timestamp: SystemTime::now(),
                },
            );
            rates
                .rates
                .get(to)
                .copied()
                .ok_or_else(|| "Rate not found in response".into())
        }
        StatusCode::FORBIDDEN => Err("API request limit exceeded".into()),
        _ => Err(format!("Error fetching exchange rate: {}", response.status()).into()),
    }
}

pub async fn fetch_all_exchange_rates(base: &str) -> Result<Rates, Box<dyn Error>> {
    let api_key = env::var("API_KEY")?;
    let api_url = format!(
        "https://api.exchangerate-api.com/v4/latest/{}?access_key={}",
        base, api_key
    );

    let response = reqwest::get(&api_url).await?;

    match response.status() {
        StatusCode::OK => Ok(response.json().await?),
        StatusCode::FORBIDDEN => Err("API request limit exceeded".into()),
        _ => Err(format!("Error fetching all exchange rates: {}", response.status()).into()),
    }
}
