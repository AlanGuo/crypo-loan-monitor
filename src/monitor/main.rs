use std::{thread, time};
use crate::engine::exchange::{types::Exchanges, Exchange};

static INTERVAL: u64 = 10_u64;

pub async fn main_loop(ex: &Exchange) -> Result<String, String> {
  loop {
    let depth_res = ex.depth().await;
    let loan_info_res = ex.loan_info().await;
    if !depth_res.is_err() && !loan_info_res.is_err() {
      let depth = depth_res.unwrap();
      let loan_info = loan_info_res.unwrap();
      let ask_price_volume = depth.tick.asks[0];
      
      log::info!("loan_info: {:#?}", loan_info);
      log::info!("{}/{}, price & volume: {:?}", ex.symbol, ex.currency, ask_price_volume);
    } else {
      if depth_res.is_err() {
        log::error!("ex.depth error: {}", depth_res.unwrap_err());
      }
      if loan_info_res.is_err() {
        log::error!("ex.loan_info error: {}", loan_info_res.unwrap_err());
      }
    }
    thread::sleep(time::Duration::from_secs(INTERVAL));
  }
}