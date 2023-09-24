use anyhow::Error;
use arbitrage_processing::{
    api::{
        order::OrderRequest,
        types::{OrderType, Side},
        BinanceRequest,
    },
    client::{config::Config, BinanceClient},
};
use clap::{App, Arg};
use env_logger::TimestampPrecision;
use log::LevelFilter;
use std::time::{Duration, Instant};

fn build_app<'help>() -> App<'help> {
    let config = Arg::new("config")
        .default_value("config.json")
        .short('c')
        .long("config")
        .takes_value(true)
        .value_name("PATH")
        .help("Path to the config");

    let amount = Arg::new("amount")
        .default_value("10")
        .short('a')
        .long("amount")
        .takes_value(true)
        .value_name("AMOUNT")
        .help("Amount of queries");

    let loglevel = Arg::new("loglevel")
        .possible_values(["off", "error", "info", "debug"])
        .default_value("info")
        .short('l')
        .long("log-level")
        .takes_value(true)
        .value_name("LEVEL")
        .help("Log level");

    App::new("Speed test").args(&[config, loglevel, amount])
}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    let start_time = Instant::now();
    let app = build_app();
    let matches = app.get_matches();
    let config_path = matches.value_of("config").unwrap();
    let loglevel = matches.value_of("loglevel").unwrap();
    let amount = matches.value_of_t("amount").unwrap();

    let loglevel = match loglevel {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        _ => unimplemented!("Unimplemented loglevel {}", loglevel),
    };

    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format_timestamp(Some(TimestampPrecision::Millis))
        .filter_level(loglevel)
        .init();

    log::info!("Client started");

    let config = Config::load(config_path).unwrap();
    let api = config.api_key;
    let secret_key = config.secret_key;
    let mut client = BinanceClient::new(config.url);
    client.connect(Duration::new(20, 0)).await;

    for _ in 0..amount {
        let mut order = OrderRequest::new(
            &api,
            "BTCUSDT",
            Side::Sell,
            OrderType::Market,
            Some("0.01".into()),
        );

        secret_key.sign(&mut order);
        client.feed(order.preprocess()?).await?;
    }

    client.flush().await?;

    while client.pending() {
        let _response = client.next().await;
    }

    client.disconnect().await?;
    log::info!("Total run time: {:?}", start_time.elapsed());

    Ok(())
}
