use std::convert::TryFrom;

use ethers::prelude::Middleware;
use ethers::providers::{Http, Provider};

pub(crate) async fn get_block_timestamp(provider: &Provider<Http>, block_number: u64) -> u64 {
    let block = provider.get_block(block_number).await.unwrap();
    u64::try_from(block.unwrap().timestamp).unwrap()
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::env;

    use chrono::{DateTime, TimeZone, Utc};
    use ethers::providers::{Http, Provider};
    use log::info;
    use util::async_result::AsyncResult;

    use crate::api_calls::get_block_timestamp;
    use crate::subscription::tests::get_eth_subscription;

    async fn test_get_block_timestamp_async() -> AsyncResult<()> {
        let subscription = get_eth_subscription();
        let result = &subscription.params.as_ref().unwrap().result;
        let block_number = result.block_number();

        let alchemy_api_key = env::var("ALCHEMY_API_KEY").unwrap();
        let alchemy_url = format!("https://eth-mainnet.alchemyapi.io/v2/{}", alchemy_api_key);
        let provider = Provider::<Http>::try_from(alchemy_url).unwrap();

        let timestamp = get_block_timestamp(&provider, block_number).await;
        let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).expect("Invalid timestamp");
        info!("Block {} timestamp: {}", block_number, datetime.format("%Y-%m-%d %H:%M:%S UTC"));
        Ok(())
    }

    #[test]
    fn test_get_block_timestamp() {
        util_test::logging::init_logging();
        tokio_test::block_on(test_get_block_timestamp_async());
    }
}
