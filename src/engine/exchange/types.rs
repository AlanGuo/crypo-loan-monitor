use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MarketStatus {
  TRADING,
  CLOSED
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum Exchanges {
  HUOBI,
  BINANCE,
  OKEX
}

impl fmt::Display for Exchanges {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Exchanges::HUOBI => write!(f, "HUOBI"),
      Exchanges::BINANCE => write!(f, "BINANCE"),
      Exchanges::OKEX => write!(f, "OKEX"),
    }
  }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OrderSide {
  BUY,
  SELL
}

impl fmt::Display for OrderSide {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      OrderSide::BUY => write!(f, "BUY"),
      OrderSide::SELL => write!(f, "SELL"),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
  NEW,
  PARTIALLYFILLED,
  FILLED,
  CANCELED,
  REJECTED,
  EXPIRED
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
  pub asks: Vec<[f64;2]>,
  pub bids: Vec<[f64;2]>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DepthInfo {
  pub tick: Tick,
  pub ts: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
  pub available_symbol: f64,
  pub frozen_symbol: f64,
  pub available_currency: f64,
  pub frozen_currency: f64
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderInfo {
  pub id: String,
  pub volume: f64,
  pub price: f64,
  pub created_at: u64,
  pub status: OrderStatus,
  pub trade_avg_price: f64,
  pub side: OrderSide,
  pub trade_volume: f64
}
#[derive(Debug)]
pub struct OpenInfo {}
#[derive(Debug)]
pub struct CloseInfo {}
#[derive(Debug)]
pub struct CancelInfo {}
#[derive(Debug)]
pub struct MarketInfo {
  pub pair: String,
  pub status: MarketStatus,
  pub price_tick: f64,
  pub contract_size: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoanInfo {
  pub symbol: String,
  pub min_volume: f64
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AccountType {
  USDSFUTURE,
  SPOT
}