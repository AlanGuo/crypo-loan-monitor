pub mod types;
pub mod config;
pub mod binance;
pub mod huobi;
pub mod okex;
use types::{ Exchanges, AccountInfo, OrderInfo, OrderSide, DepthInfo, LoanInfo };

#[derive(Debug, Clone)]
pub struct Exchange {
  pub name: Exchanges, // 交易所的名称标识
  pub symbol: String, //对应的合约标的,  BTC, ETH ...
  pub currency: String, //合约本位货币
  pub host: String, // api host
  pub protocol: String,
  pub config: String // 配置名称
}

impl Exchange {
  pub async fn depth(&self) -> Result<DepthInfo, String> {
    match self.name {
      Exchanges::HUOBI => {
        return Err(format!("{:?}::depth not_implemented", self.name));
      }
      Exchanges::BINANCE => {
        return binance::depth(self).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::depth not_implemented", self.name));
      }
    }
  }
  pub async fn account_info(&self) -> Result<AccountInfo, String> {
    match self.name {
      Exchanges::HUOBI => {
        return huobi::account_info(self).await;
      }
      Exchanges::BINANCE => {
        return binance::account_info(self).await;
      }
      Exchanges::OKEX => {
        return okex::account_info(self).await;
      }
    }
  }

  pub async fn order_info(&self, order_id: String) -> Result<OrderInfo, String> {
    match self.name {
      Exchanges::HUOBI => {
        return Err(format!("{:?}::order_info not_implemented", self.name));
      }
      Exchanges::BINANCE => {
        return binance::order_info(self, order_id).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::order_info not_implemented", self.name))
      }
    }
  }

  pub async fn create_order(&self, side: OrderSide, price: f64, volume: f64) -> Result<String, String> {
    match self.name {
      Exchanges::HUOBI => {
        return Err(format!("{:?}::create_order not_implemented", self.name));
      }
      Exchanges::BINANCE => {
        return binance::create_order(self, side, price, volume).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::create_order not_implemented", self.name));
      }
    }
  }
  pub async fn cancel_order(&self, order_id: String) -> Result<bool, String> {
    match self.name {
      Exchanges::HUOBI => {
        return Err(format!("{:?}::cancel_order not_implemented", self.name));
      }
      Exchanges::BINANCE => {
        return binance::cancel_order(self, order_id).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::cancel_order not_implemented", self.name));
      }
    }
  }
  pub async fn cancel_all_order(&self) -> Result<bool, String> {
    match self.name {
      Exchanges::HUOBI => {
        return Err(format!("{:?}::cancel_all_order not_implemented", self.name));
      }
      Exchanges::BINANCE => {
        return binance::cancel_all_order(self).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::cancel_all_order not_implemented", self.name));
      }
    }
  }
  pub async fn loan_info(&self) -> Result<LoanInfo, String> {
    match self.name {
      Exchanges::HUOBI => {
        return huobi::loan_info(self).await;
      }
      Exchanges::BINANCE => {
        return binance::loan_info(self).await;
      }
      Exchanges::OKEX => {
        return Err(format!("{:?}::loan_info desperated", self.name));
        // return okex::loan_info(self).await;
      }
    }
  }
  pub async fn withdraw(&self, asset: String, address: String, amount: f64) -> Result<String, String> {
    match self.name {
      Exchanges::HUOBI => {
        return huobi::withdraw(self, asset, address, amount).await;
      }
      Exchanges::BINANCE => {
        return binance::withdraw(self, asset, address, amount).await;
      }
      Exchanges::OKEX => {
        return okex::withdraw(self, asset, address, amount).await;
      }
    }
  }
}