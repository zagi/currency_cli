use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct ApiResponse {
    rate: f64,
}

async fn fetch_exchange_rate(from: &str, to: &str) -> Result<f64, reqwest::Error> {
    let api_url = format!("https://api.exchangerate-api.com/v4/latest/{}", from);
    let response = reqwest::get(api_url).await?.json::<ApiResponse>().await?;
    Ok(response.rate)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: currency_converter <from_currency> <to_currency> <amount>");
        return;
    }

    let from_currency = &args[1];
    let to_currency = &args[2];
    let amount: f64 = args[3].parse().expect("Invalid amount");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        match fetch_exchange_rate(from_currency, to_currency).await {
            Ok(rate) => {
                let converted_amount = amount * rate;
                println!("{} {} is {} {}", amount, from_currency, converted_amount, to_currency);
            }
            Err(e) => eprintln!("Error fetching exchange rate: {}", e),
        }
    });
}
