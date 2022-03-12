use std::{collections::HashMap};
use super::config::{ OkexConfig, OKEX_USDT_WITHDRAW_CHAIN };
use super::types::{ MarketInfo, MarketStatus, DepthInfo, Tick, AccountInfo, OrderInfo, OrderStatus, OrderSide, LoanInfo };
use serde_json::{ Value };
use base64::{ encode };
use sha2::{Sha256};
use hmac::{Hmac, Mac, NewMac};
use chrono::offset::Utc;
use chrono::{ DateTime, NaiveDateTime };
use super::Exchange;
use crate::util::handle_body;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// for okex sign
async fn build_okex_sign(cfg: &OkexConfig, protocol: &str, host: &str, method: &str, path: &str, body: Vec<[&str;2]>, is_array: bool) -> Result<(String, String, String), String> {
  // get server time
  let full_url = format!("{}://{}/api/v5/public/time", protocol, host);
  let body_resp = reqwest::get(&full_url).await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  let ts_str: String = json_resp["data"][0]["ts"].as_str().expect("read ts error").to_string();
  let timestamp: i64 = ts_str.parse::<i64>().expect("parse time_str error");
  let dt = NaiveDateTime::from_timestamp(timestamp / 1000, (timestamp as f64 % 1000_f64) as u32 * 1_000_000);
  let datetime: DateTime<Utc> = DateTime::from_utc(dt, Utc);
  let mut body_serilizer = if body.len() > 0 && !is_array { String::from("{") } else { String::from("") };
  for (i, item) in body.iter().enumerate() {
    if i == body.len() - 1 {
      // 最后一项
      if item[1].chars().nth(0).unwrap() == '[' {
        if item[0] == "" {
          body_serilizer += item[1];
        } else {
          body_serilizer += &format!("\"{}\":{}}}", item[0], item[1]);
        }
      } else {
        body_serilizer += &format!("\"{}\":\"{}\"}}", item[0], item[1]);
      }
    } else {
      // 非最后一项
      if item[1].chars().nth(0).unwrap() == '[' {
        if item[0] == "" {
          body_serilizer += item[1];
        } else {
          body_serilizer += &format!("\"{}\":{},", item[0], item[1]);
        }
      } else {
        body_serilizer += &format!("\"{}\":\"{}\",", item[0], item[1]);
      }
    }
  }
  let time_str = datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
  let full_str = format!("{}{}{}{}", time_str, method.to_uppercase(), path, body_serilizer);
  let mut mac = HmacSha256::new_varkey(cfg.secret_key.as_bytes()).expect("HMAC can take key of any size");
  mac.update(full_str.as_bytes());
  let signature = encode(mac.finalize().into_bytes());
  return Ok((
    signature.to_string(),
    time_str,
    body_serilizer
  ));
}

// api desprated
pub async fn loan_info(ex: &Exchange) -> Result<LoanInfo, String> {
  let cfg: OkexConfig = confy::load(&ex.config).expect("read okex config error");
  let path = format!("/api/margin/v3/accounts/{}-{}/availability", ex.symbol.to_uppercase(), ex.currency.to_uppercase());
  let (sign, timestamp, _body) = build_okex_sign(&cfg, &ex.protocol, &ex.host, "GET", &path, [].to_vec(), false).await?;
  let full_url = format!("{}://{}{}", ex.protocol, ex.host, path);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str())
  .header("OK-ACCESS-KEY", cfg.access_id)
  .header("OK-ACCESS-SIGN", sign)
  .header("OK-ACCESS-TIMESTAMP", timestamp)
  .header("OK-ACCESS-PASSPHRASE", cfg.passphrase)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  let key = format!("currency:{}", ex.symbol.to_uppercase());
  if json_resp.is_array() {
    let asset_item: &Value = &json_resp[0];
    if !asset_item[key].is_null() {
      let li = LoanInfo {
        symbol: ex.symbol.clone(),
        min_volume: 0_f64,
      };
      return Ok(li);
    } else {
      return Err(format!("no loan info for {}", ex.symbol.clone()));
    }
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}

// 查询资金账户，OKEX还有个交易账户
pub async fn account_info(ex: &Exchange) -> Result<AccountInfo, String> {
  let cfg: OkexConfig = confy::load(&ex.config).expect("read okex config error");
  let path = format!("/api/v5/asset/balances?ccy={},{}", ex.symbol.to_uppercase(), ex.currency.to_uppercase());
  let (sign, timestamp, _body) = build_okex_sign(&cfg, &ex.protocol, &ex.host, "GET", &path, [].to_vec(), false).await?;
  let full_url = format!("{}://{}{}", ex.protocol, ex.host, path);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str())
  .header("OK-ACCESS-KEY", cfg.access_id)
  .header("OK-ACCESS-SIGN", sign)
  .header("OK-ACCESS-TIMESTAMP", timestamp)
  .header("OK-ACCESS-PASSPHRASE", cfg.passphrase)
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp["code"] == "0" {
    let arr= json_resp["data"].as_array().expect("read details error");
    let symbol_item_opt = arr.iter().find(|x| x["ccy"] == ex.symbol.to_uppercase());
    let currency_item_opt = arr.iter().find(|x| x["ccy"] == ex.currency.to_uppercase());
    let mut available_symbol = 0_f64;
    let mut frozen_symbol = 0_f64;
    let mut available_currency = 0_f64;
    let mut frozen_currency = 0_f64;
    if symbol_item_opt.is_some() {
      let symbol_item = symbol_item_opt.unwrap();
      available_symbol = symbol_item["availBal"].as_str().expect("read availBal error").parse::<f64>().expect("parse availBal error");
      frozen_symbol = symbol_item["frozenBal"].as_str().expect("read frozenBal error").parse::<f64>().expect("parse frozenBal error");
    }
    if currency_item_opt.is_some() {
      let currency_item = currency_item_opt.unwrap();
      available_currency = currency_item["availBal"].as_str().expect("read availBal error").parse::<f64>().expect("parse availBal error");
      frozen_currency = currency_item["frozenBal"].as_str().expect("read frozenBal error").parse::<f64>().expect("parse frozenBal error");
    }
    let ai = AccountInfo {
      available_symbol,
      frozen_symbol,
      available_currency,
      frozen_currency,
    };
    return Ok(ai);
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}

pub async fn withdraw(ex: &Exchange, asset: String, address: String, amount: f64) -> Result<String, String> {
  let cfg: OkexConfig = confy::load(&ex.config).expect("read okex config error");
  let mut currency = asset.clone();
  if asset.to_uppercase().eq("USDT") {
    currency = String::from(OKEX_USDT_WITHDRAW_CHAIN);
  }
  // get fee
  let path = format!("/api/v5/asset/currencies");
  let (sign, timestamp, _body) = build_okex_sign(&cfg, &ex.protocol, &ex.host, "GET", &path, [].to_vec(), false).await?;
  let full_url = format!("{}://{}{}", ex.protocol, ex.host, path);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str())
  .header("OK-ACCESS-KEY", cfg.access_id.clone())
  .header("OK-ACCESS-SIGN", sign)
  .header("OK-ACCESS-TIMESTAMP", timestamp)
  .header("OK-ACCESS-PASSPHRASE", cfg.passphrase.clone())
  .send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp["code"] == "0" {
    let asset_item: &Value = json_resp["data"].as_array().expect("json_resp['data'] as_array error").into_iter()
    .find(|x| x["ccy"].as_str().expect("read ccy error").eq(&asset.to_uppercase()) &&x["chain"].as_str().expect("read chain error").eq(&currency)).expect("find chain error");
    let fee_str = asset_item["minFee"].as_str().expect("read minFee error");
    let path = "/api/v5/asset/withdrawal";
    println!("fee_str: {}", fee_str);
    let (sign, timestamp, body) = build_okex_sign(&cfg, &ex.protocol, &ex.host, "POST", &path, [
      ["amt", &amount.to_string()],
      ["ccy", &asset.to_uppercase()],
      ["chain", &currency],
      ["dest", "4"],
      ["toAddr", &address],
      ["pwd", &cfg.trade_pwd],
      ["fee", &fee_str]
    ].to_vec(), false).await?;
    let full_url = format!("{}://{}{}", ex.protocol, ex.host, path);
    let client = reqwest::Client::new();
    let req = client.post(full_url.as_str())
    .header("OK-ACCESS-KEY", cfg.access_id.clone())
    .header("OK-ACCESS-SIGN", sign)
    .header("OK-ACCESS-TIMESTAMP", timestamp)
    .header("OK-ACCESS-PASSPHRASE", cfg.passphrase.clone())
    .header("Content-Type", "application/json; charset=utf-8")
    .body(body);
    let body_resp = req.send().await;
    let body_text = handle_body(body_resp).await?;
    let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
    if json_resp["code"] == "0" {
      let data_item = &json_resp["data"][0];
      let id = data_item["wdId"].as_str().expect("read wdId error"); 
      return Ok(String::from(id));
    } else {
      return Err(format!("{}: {:?}", ex.name, json_resp));
    }
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}