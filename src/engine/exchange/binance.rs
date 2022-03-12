 
use std::{collections::HashMap};
use super::config::{ BinanceConfig, BINANCE_USDT_WITHDRAW_CHAIN };
use super::types::{ Tick, AccountInfo, OrderInfo, OrderStatus, OrderSide, DepthInfo, LoanInfo };
use serde_json::{ Value };
use sha2::{Sha256};
use hmac::{Hmac, Mac, NewMac};
use url::form_urlencoded::Serializer;
use super::Exchange;
use crate::util::handle_body;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// for binance sign
pub async fn build_binance_sign(cfg: &BinanceConfig, protocol: &str, host: &str, params: Vec<[&str;2]>, body: Vec<[&str;2]>) -> Result<String, String> {
  // get server time
  let full_url = format!("{}://{}/api/v3/time", protocol, host);
  let body_resp = reqwest::get(&full_url).await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  let timestamp_str = json_resp["serverTime"].to_string();

  let mut all_params: Vec<[&str;2]> = Vec::new();
  all_params.clone_from(&params);
  if body.len() == 0 {
    all_params.push(["recvWindow", "5000"]);
    all_params.push(["timestamp", timestamp_str.as_str()]);
  }
  let mut param_serilizer: Serializer<String> = Serializer::new(String::new());
  for item in all_params.into_iter() {
    param_serilizer.append_pair(item[0], item[1]);
  }
  let param_str = param_serilizer.finish();
  // body params
  let mut all_body: Vec<[&str;2]> = Vec::new();
  all_body.clone_from(&body);
  if body.len() > 0 {
    all_body.push(["recvWindow", "5000"]);
    all_body.push(["timestamp", timestamp_str.as_str()]);
  }
  let mut body_serilizer: Serializer<String> = Serializer::new(String::new());
  for item in all_body.into_iter() {
    body_serilizer.append_pair(item[0], item[1]);
  }
  let body_str = body_serilizer.finish();
  let full_str = format!("{}{}", param_str, body_str);
  let mut mac = HmacSha256::new_varkey(cfg.secret_key.as_bytes()).expect("HMAC can take key of any size");
  mac.update(full_str.as_bytes());
  let sign_bytes = mac.finalize().into_bytes();
  let signature_str = hex::encode(sign_bytes.as_slice());

  return Ok(param_str + "&signature=" + signature_str.as_str());
}

pub async fn depth(ex: &Exchange) -> Result<DepthInfo, String> {
  let full_url = format!("{}://{}/api/v3/depth?symbol={}{}&limit=5", ex.protocol, ex.host, ex.symbol.to_uppercase(), ex.currency.to_uppercase());
  let body_resp = reqwest::get(full_url.as_str()).await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["asks"].is_null() {
    let asks = json_resp["asks"].as_array().expect("no asks");
    let bids = json_resp["bids"].as_array().expect("no bids");
    let mut asks_vec: Vec<[f64;2]> = Vec::new();
    let mut bids_vec: Vec<[f64;2]> = Vec::new();
    // 取5个深度
    for i in 0..5 {
      asks_vec.push([asks[i][0].as_str().expect("asks as_f64 error").parse::<f64>().expect("asks str -> f64 error"),
      asks[i][1].as_str().expect("asks as_f64 error").parse::<f64>().expect("str -> f64 error")]);
      bids_vec.push([bids[i][0].as_str().expect("bids as_f64 error").parse::<f64>().expect("bids str -> f64 error"),
      bids[i][1].as_str().expect("bids as_f64 error").parse::<f64>().expect("bids str -> f64 error")]);
    }
    let di = DepthInfo {
      tick: Tick {
        asks: asks_vec,
        bids: bids_vec
      },
      // fake data
      ts: 0_i64,
    };
    return Ok(di);
  } else {
    return Err(format!("{}", json_resp));
  }
}

pub async fn account_info(ex: &Exchange) -> Result<AccountInfo, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/api/v3/account?{}", ex.protocol, ex.host, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["balances"].is_null() {
    let balances = json_resp["balances"].as_array().expect("no balances");
    let symbol_item = balances.into_iter().find(|x| x["asset"] == format!("{}", ex.symbol.to_uppercase())).expect("find symbol item error");
    let currency_item = balances.into_iter().find(|x| x["asset"] == format!("{}", ex.currency.to_uppercase())).expect("find currency item error");
    let ai = AccountInfo {
      available_symbol: symbol_item["free"].as_str().expect("read free symbol error").parse::<f64>().expect("parse free symbol error"),
      frozen_symbol: symbol_item["locked"].as_str().expect("read locked symbol error").parse::<f64>().expect("parse locked symbol error"),
      available_currency: currency_item["free"].as_str().expect("read free currency error").parse::<f64>().expect("parse free currency error"),
      frozen_currency: currency_item["locked"].as_str().expect("read locked currency error").parse::<f64>().expect("parse locked currency error")
    };
    return Ok(ai);
  } else {
    return Err(format!("{}", json_resp));
  }
}

pub async fn order_info(ex: &Exchange, order_id: String) -> Result<OrderInfo, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [
    ["symbol", &format!("{}{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase())],
    ["orderId", order_id.as_str()]
  ].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/api/v3/order?{}", ex.protocol, ex.host, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let obj: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  let mut order_status_map = HashMap::new();
  order_status_map.insert("NEW", OrderStatus::NEW);
  order_status_map.insert("PARTIALLY_FILLED", OrderStatus::PARTIALLYFILLED);
  order_status_map.insert("FILLED", OrderStatus::FILLED);
  order_status_map.insert("CANCELED", OrderStatus::CANCELED);
  order_status_map.insert("REJECTED", OrderStatus::REJECTED);
  order_status_map.insert("EXPIRED", OrderStatus::EXPIRED);
  let mut side_map = HashMap::new();
  side_map.insert("BUY", OrderSide::BUY);
  side_map.insert("SELL", OrderSide::SELL);
  if obj["orderId"].is_null() {
    return Err(format!("{:?}", obj));
  } else {
    let oi = OrderInfo {
      id: obj["orderId"].as_u64().expect("read orderId error").to_string(),
      volume: obj["origQty"].as_str().expect("read origQty error").parse::<f64>().expect("parse origQty error"),
      price: obj["price"].as_str().expect("read price error").parse::<f64>().expect("parse price error"),
      status: order_status_map[obj["status"].as_str().expect("read status err")].to_owned(),
      side: side_map[obj["side"].as_str().expect("read side err")].to_owned(),
      created_at: obj["time"].as_u64().expect("read created_at err"),
      trade_volume: obj["executedQty"].as_str().expect("read executedQty error").parse::<f64>().expect("parse executedQty err"),
      trade_avg_price: 0_f64
    };
    return Ok(oi);
  }
}

pub async fn create_order(ex: &Exchange, side: OrderSide, price: f64, volume: f64) -> Result<String, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [
    ["symbol", &format!("{}{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase())],
    ["side", &String::from(side.to_string())],
    ["type", "LIMIT"],
    ["timeInForce", "GTC"],
    ["quantity", &volume.to_string()],
    ["price", &price.to_string()]
  ].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/api/v3/order?{}",
  ex.protocol,
  ex.host,
  param_str);
  let client = reqwest::Client::new();
  let body_resp = client.post(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["orderId"].is_null() {
    return Ok(String::from(json_resp["orderId"].as_u64().expect("no orderId").to_string()));
  } else {
    return Err(format!("{:?}", json_resp));
  }
}

pub async fn cancel_order(ex: &Exchange, order_id: String) -> Result<bool, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [
    ["symbol", &format!("{}{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase())],
    ["orderId", &order_id],
  ].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/api/v3/order?{}", ex.protocol, ex.host, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.delete(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp["status"].is_null() {
    return Err(format!("{:?}", json_resp));
  } else {
    return Ok(json_resp["status"].as_str().expect("no status") == "CANCELED");
  }
}

pub async fn cancel_all_order(ex: &Exchange) -> Result<bool, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [
    ["symbol", &format!("{}{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase())],
  ].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/api/v3/openOrders?{}", ex.protocol, ex.host, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.delete(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send()
  .await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["code"].is_null() {
    return Err(format!("{:}", json_resp));
  } else {
    let res_arr = json_resp.as_array().expect("as_array error");
    let mut all_cancelled = true;
    for item in res_arr.iter() {
      if item["status"].as_str().expect("no status") != "CANCELED" {
        all_cancelled = false;
        log::warn!("cancel failed: {:}", item);
      }
    }
    return Ok(all_cancelled);
  }
}

pub async fn loan_info(ex: &Exchange) -> Result<LoanInfo, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [[
    "symbol", &format!("{}{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase())
  ]].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/sapi/v1/margin/isolated/pair?{}", ex.protocol, ex.host, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp.is_object() {
    let asset_item = json_resp;
    if asset_item["code"].is_null() {
      let is_margin_res = asset_item["isMarginTrade"].as_bool();
      if is_margin_res.is_some() {
        let is_margin = is_margin_res.unwrap();
        if is_margin {
          let li = LoanInfo {
            symbol: ex.symbol.clone(),
            min_volume: 0_f64
          };
          return Ok(li);
        } else {
          return Err(format!("no loan info for {}", ex.symbol.clone()));
        }
      } else {
        return Err(format!("no loan info for {}", ex.symbol.clone()));
      }
    } else {
      return Err(format!("{}: {:?}", ex.name, asset_item));
    }
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}

pub async fn withdraw(ex: &Exchange, asset: String, address: String, amount: f64) -> Result<String, String> {
  let cfg: BinanceConfig = confy::load(&ex.config).expect("read binance config error");
  let network: String = if asset.eq("usdt") {String::from(BINANCE_USDT_WITHDRAW_CHAIN)} else { asset.clone() };
  let param_str = build_binance_sign(&cfg, &ex.protocol, &ex.host, [
    ["coin", &asset],
    ["address", &address],
    ["network", &network],
    ["amount", &amount.to_string()]
  ].to_vec(), [].to_vec()).await?;
  let full_url = format!("{}://{}/sapi/v1/capital/withdraw/apply?{}",
  ex.protocol,
  ex.host,
  param_str);
  let client = reqwest::Client::new();
  let body_resp = client.post(full_url.as_str()).header("X-MBX-APIKEY", cfg.access_id)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  println!("{}", body_text);
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["id"].is_null() {
    return Ok(String::from(json_resp["id"].as_str().expect("read id error")));
  } else {
    return Err(format!("{:?}", json_resp));
  }
}