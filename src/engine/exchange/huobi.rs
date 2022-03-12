use super::config::{ HuobiConfig, HUOBI_USDT_WITHDRAW_CHAIN };
use super::types::{ LoanInfo, AccountInfo, AccountType };
use serde_json::{ Value };
use std::{collections::HashMap};
use base64::{ encode };
use sha2::{Sha256};
use hmac::{Hmac, Mac, NewMac};
use chrono::offset::Utc;
use chrono::{ DateTime, NaiveDateTime };
use url::form_urlencoded::Serializer;
use super::Exchange;
use crate::util::{ handle_body, huobi_withdraw_fee };

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

// for huobi sign
pub async fn build_huobi_sign(cfg: &HuobiConfig, protocol: &str, host: &str,  timestamp_host: &str, method: &str, path: &str, params: Vec<[&str;2]>) -> Result<String, String> {
  let mut all_params: Vec<[&str;2]> = Vec::new();
  // get server time
  let full_url = format!("{}://{}/v1/common/timestamp", protocol, timestamp_host);
  let body_resp = reqwest::get(&full_url).await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  let timestamp: i64 = json_resp["data"].as_i64().expect("read ts error");
  let dt = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
   // Create a normal DateTime from the NaiveDateTime
  let datetime: DateTime<Utc> = DateTime::from_utc(dt, Utc);
  all_params.clone_from(&params);
  all_params.push(["AccessKeyId", cfg.access_id.as_str()]);
  all_params.push(["SignatureMethod", cfg.signature_method.as_str()]);
  all_params.push(["SignatureVersion", cfg.signature_version.as_str()]);
  let time_str = datetime.format("%Y-%m-%dT%H:%M:%S").to_string();
  all_params.push(["Timestamp", &time_str]);
  all_params.sort_by(|a, b| a[0].cmp(b[0]));
  let mut serilizer: Serializer<String> = Serializer::new(String::new());
  for item in all_params.into_iter() {
    serilizer.append_pair(item[0], item[1]);
  }
  let param_str = serilizer.finish();
  let full_str = format!("{}\n{}\n{}\n{}", method.to_uppercase(), host, path, param_str);
  let mut mac = HmacSha256::new_varkey(cfg.secret_key.as_bytes()).expect("HMAC can take key of any size");
  mac.update(full_str.as_bytes());
  let signature = encode(mac.finalize().into_bytes());
  let mut serilizer: Serializer<String> = Serializer::new(String::new());
  serilizer.append_pair("Signature", signature.as_str());
  return Ok(param_str + "&" + &serilizer.finish());
}

pub async fn account_info(ex: &Exchange) -> Result<AccountInfo, String> {
  let cfg: HuobiConfig = confy::load(&ex.config).expect("read huobi config error");
  let param_str = build_huobi_sign(&cfg, &ex.protocol, &ex.host, &ex.host, "GET", &format!("/v1/account/accounts/{}/balance", cfg.account_id),
  [].to_vec()).await?;
  let full_url = format!("{}://{}/v1/account/accounts/{}/balance?{}", ex.protocol, ex.host, cfg.account_id, param_str);
  let client = reqwest::Client::new();
  let body_resp = client.get(full_url.as_str()).send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp["status"] == "ok" {
    let obj = &json_resp["data"];
    let list = obj["list"].as_array().expect("read list error");
    let currency_available_item = list.into_iter().find(|x| x["currency"] == ex.currency && x["type"] == "trade").expect("read trade currency error");
    let currency_frozen_item = list.into_iter().find(|x| x["currency"] == ex.currency && x["type"] == "frozen").expect("read frozen currency error");
    let symbol_available_item = list.into_iter().find(|x| x["currency"] == ex.symbol && x["type"] == "trade").expect("read trade symbol error");
    let symbol_frozen_item = list.into_iter().find(|x| x["currency"] == ex.symbol && x["type"] == "frozen").expect("read frozen symbol error");

    let ai = AccountInfo {
      available_symbol: symbol_available_item["balance"].as_str().expect("read symbol_available_item balance error").parse::<f64>().expect("parse symbol_available_item balance error"),
      frozen_symbol: symbol_frozen_item["balance"].as_str().expect("read symbol_frozen_item balance error").parse::<f64>().expect("parse symbol_frozen_item balance error"),
      available_currency: currency_available_item["balance"].as_str().expect("read currency_available_item balance error").parse::<f64>().expect("parse currency_available_item balance error"),
      frozen_currency: currency_frozen_item["balance"].as_str().expect("read currency_frozen_item balance error").parse::<f64>().expect("parse currency_frozen_item balance error"),
    };
    return Ok(ai);
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}


pub async fn loan_info(ex: &Exchange) -> Result<LoanInfo, String> {
  let cfg: HuobiConfig = confy::load(&ex.config).expect("read huobi config error");
  let symbols = format!("{}{}", ex.symbol.to_lowercase(), ex.currency.to_lowercase());
  let param_str = build_huobi_sign(&cfg, &ex.protocol, &ex.host, &ex.host, "GET", "/v1/margin/loan-info",
  [["symbols", &symbols]].to_vec()).await?;
  let full_url = format!("{}://{}/v1/margin/loan-info?{}", ex.protocol, ex.host, param_str);
  let body_resp = reqwest::get(full_url.as_str()).await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if json_resp["status"] == "ok" {
    let arr = json_resp["data"].as_array().expect("data as array error");
    if arr.len() > 0 { 
      let loan_arr = arr[0]["currencies"].as_array().expect("currencies as array error");
      let li = LoanInfo {
        symbol: ex.symbol.clone(),
        min_volume: loan_arr[0]["min-loan-amt"].as_str().expect("read min-loan-amt err").parse::<f64>().expect("parse min-loan-amt error"),
      };
      return Ok(li);
    } else {
      return Err(format!("no loan info for {}", ex.symbol.clone()));
    }
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}

pub async fn withdraw(ex: &Exchange, asset: String, address: String, amount: f64) -> Result<String, String> {
  let cfg: HuobiConfig = confy::load(&ex.config).expect("read huobi config error");
  let param_str = build_huobi_sign(&cfg, &ex.protocol, &ex.host, &ex.host, "POST", "/v1/dw/withdraw/api/create",
  [].to_vec()).await?;
  let full_url = format!("{}://{}/v1/dw/withdraw/api/create?{}", ex.protocol, ex.host, param_str);
  let network: String = if asset.eq("usdt") { String::from(HUOBI_USDT_WITHDRAW_CHAIN) } else { asset.clone() };
  let fee = huobi_withdraw_fee(&asset);
  let client = reqwest::Client::new();
  let mut map = HashMap::new();
  map.insert("address", address.clone());
  map.insert("amount", (amount - fee).to_string());
  map.insert("currency", asset.clone());
  map.insert("chain", network.clone());
  map.insert("fee", fee.to_string());
  let body_resp = client.post(full_url.as_str()).json(&map).send().await;
  let body_text = handle_body(body_resp).await?;
  let json_resp: Value = serde_json::from_str(body_text.as_str()).expect("json parse error");
  if !json_resp["data"].is_null() {
    let id = json_resp["data"].as_u64().expect("read data error").to_string();
    return Ok(String::from(id));
  } else {
    return Err(format!("{}: {:?}", ex.name, json_resp));
  }
}