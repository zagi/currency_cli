# Currency Converter Tool

This tool is a simple command-line application written in Rust that converts amounts between different currencies using real-time exchange rate data fetched from an API.

## Prerequisites

To build and run this tool, you need:

- Rust programming environment ([Installation guide](https://www.rust-lang.org/tools/install))
- `cargo` (Rust's package manager, comes with Rust installation)
- An API key from [ExchangeRate-API](https://www.exchangerate-api.com/)

## Obtaining API Keys

1. Visit [ExchangeRate-API](https://www.exchangerate-api.com/).
2. Sign up for an account and choose the appropriate plan.
3. Once logged in, navigate to the dashboard to find your API key.

## Setting Up the Project

Clone the repository to your local machine:

```bash
git clone git@github.com:zagi/currency_cli.git
cd currency_cli
```

Set up your API key:

1. Create a `.env` file in the root of the project directory.
2. Add your API key to the `.env` file:

```
API_KEY=your_api_key_here
```

## Building the Project

Navigate to the project directory and use `cargo` to build the project:

```bash
cargo build --release
```

## Running the Tool

After building, you can run the tool using `cargo`:

```bash
cargo run -- <from_currency> <to_currency> <amount>
```

Or directly from the executable:

```bash
./target/release/currency <from_currency> <to_currency> <amount>
```

To list all available currencies and their current exchange rates(base_currency is optional, if not provided it will be PLN):

```bash
cargo run -- list <base_currency>
```
Or 

```bash
./target/release/currency list <base_currency>
```

The clap provides easy help:
```bash
./target/release/currency --help
```

or for list subcommand:
```bash
./target/release/currency list --help
```

## Running with Docker

To build the Docker image, run the following command in the project's root directory:

```bash
docker build -t currency .
```

To run the application using Docker, execute:
```bash
docker run --rm currency currency <from_currency> <to_currency> <amount>
```

For listing all available currencies and their current exchange rates using Docker:
```bash
docker run --rm currency currency list <base_currency>
```

Note: I use --rm flag to avoid creating unnecessary containers :) 

The clap provides easy help also when running from docker image:
```bash
docker run --rm currency currency --help
```

or for list subcommand:
```bash
docker run --rm currency currency list --help
```

## Testing

Run the unit tests with the following command:

```bash
cargo test
```
