mod engine;
mod log_util;
mod util;
mod config;
mod monitor;
use engine::exchange::{types::Exchanges, Exchange};
use monitor::main::main_loop;


#[tokio::main]
// This is the main function
async fn main() {
  log_util::init_log();

  // load exchange instance
  let binance_exchange: Exchange = Exchange {
    config: String::from("binance.18520833073"),
    name: Exchanges::BINANCE,
    symbol: String::from("crv"),
    currency: String::from("usdt"),
    host: String::from("api.binance.com"),
    protocol: String::from("https"),
  };

  let res = main_loop(&binance_exchange).await;
  if res.is_err() {
    log::error!("{}", res.unwrap_err());
  }
}
