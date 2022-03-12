pub fn min_f64 (a: f64, b: f64) -> f64 {
  return if a > b { b } else { a }
}

pub fn max_f64 (a: f64, b: f64) -> f64 {
  return if a > b { a } else { b }
}

pub async fn handle_body (body_resp: Result<reqwest::Response, reqwest::Error> ) -> Result<String, String> {
  if body_resp.is_err() {
    return Err(format!("[REQWEST ERROR]: {:}", body_resp.err().unwrap()));
  } else {
    return Ok(body_resp.unwrap().text().await.expect("body_resp as text error"));
  }
}

pub fn huobi_withdraw_fee (asset: &String) -> f64 {
 if asset.eq("usdt") {
   return 1_f64;
 } else {
   return 0_f64;
 }
}