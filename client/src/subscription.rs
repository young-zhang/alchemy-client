use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResult {
   pub address: String,
   pub topics: Vec<String>,
   pub data: String,
   #[serde(rename = "blockNumber")]
   pub block_number: String,
   #[serde(rename = "transactionHash")]
   pub transaction_hash: String,
   #[serde(rename = "transactionIndex")]
   pub transaction_index: String,
   #[serde(rename = "blockHash")]
   pub block_hash: String,
   #[serde(rename = "logIndex")]
   pub log_index: String,
   pub removed: bool,
}

impl SubscriptionResult {
   pub fn block_number(&self) -> u64 {
      let block_str = self.block_number.trim_start_matches("0x");
      u64::from_str_radix(block_str, 16).unwrap()
   }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumSubscription {
   pub id: Option<i64>,
   #[serde(rename = "jsonrpc")]
   pub json_rpc: String,
   pub result: Option<String>,
   pub method: Option<String>,
   pub params: Option<Params>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
   pub result: SubscriptionResult,
   pub subscription: String,
}

pub fn parse_ethereum_subscription(json: &str) -> Result<EthereumSubscription, serde_json::Error> {
   serde_json::from_str(json)
}

#[cfg(test)]
pub(crate) mod tests {
   use super::*;

   pub fn get_eth_subscription() -> EthereumSubscription {
      let json = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"address":"0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640","topics":["0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67","0x0000000000000000000000001bf621aa9cee3f6154881c25041bb39aed4ca7cc","0x0000000000000000000000001bf621aa9cee3f6154881c25041bb39aed4ca7cc"],"data":"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffd186cd4d8a00000000000000000000000000000000000000000000000342fa0629f92054aa00000000000000000000000000000000000043cea71fe9c91c000000563a479d0000000000000000000000000000000000000000000000015d9510b7629da88b000000000000000000000000000000000000000000000000000000000002faae","blockNumber":"0x137b559","transactionHash":"0xa31a469b2e55f2aa10df7c25b01e4ce33053908d3e3bfe6ab44c353d5223218b","transactionIndex":"0x6","blockHash":"0xf41b2a06635c8e55044610e90261feda7d093d7a443ebb98298bbac6e4f61c59","logIndex":"0x28","removed":false},"subscription":"0x25f61a1668c04b900ed86a4488778f83"}}"#;
      let sub = parse_ethereum_subscription(json).unwrap();
      sub
   }

   #[test]
   fn test_parse_ethereum_subscription() {
      let subscription = get_eth_subscription();
      assert_eq!(subscription.json_rpc, "2.0");
      assert_eq!(subscription.method.as_ref().unwrap(), "eth_subscription");
      assert_eq!(subscription.params.as_ref().unwrap().subscription, "0x25f61a1668c04b900ed86a4488778f83");

      let result = &subscription.params.unwrap().result;
      let block_number = result.block_number();
      assert_eq!(block_number, 20_428_121u64);
   }
}
