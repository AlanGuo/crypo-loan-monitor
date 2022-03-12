use serde::{Deserialize, Serialize};

pub static HUOBI_USDT_WITHDRAW_CHAIN: &str = "trc20usdt";
pub static OKEX_USDT_WITHDRAW_CHAIN: &str = "USDT-TRC20";
pub static BINANCE_USDT_WITHDRAW_CHAIN: &str = "trx";

#[derive(Serialize, Deserialize, Debug)]
pub struct HuobiConfig {
  pub access_id: String,
  pub secret_key: String,
  pub account_id: String,
  pub signature_method: String,
  pub signature_version: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct BinanceConfig {
  pub access_id: String,
  pub secret_key: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OkexConfig {
  pub access_id: String,
  pub secret_key: String,
  pub trade_pwd: String,
  pub passphrase: String
}

// `HuobiConfig` implements `Default`
impl ::std::default::Default for HuobiConfig {
  fn default() -> Self {
    Self {
      access_id: String::from(""),
      secret_key: String::from(""),
      account_id: String::from(""),
      signature_method: String::from(""),
      signature_version: String::from(""),
    }
  }
}

impl ::std::default::Default for BinanceConfig {
  fn default() -> Self {
    Self {
      access_id: String::from(""),
      secret_key: String::from("")
    }
  }
}

impl ::std::default::Default for OkexConfig {
  fn default() -> Self {
    Self {
      access_id: String::from(""),
      secret_key: String::from(""),
      trade_pwd: String::from(""),
      passphrase: String::from("")
    }
  }
}
