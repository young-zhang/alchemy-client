use std::convert::TryFrom;
use std::sync::Arc;

use ethers::contract::abigen;
use ethers::prelude::{Address, Middleware};
use ethers::providers::{Http, Provider};
use util::prelude::AsyncResult;

pub(crate) async fn get_block_timestamp(block_number: u64, provider: Arc<Provider<Http>>) -> u64 {
    let block = provider.get_block(block_number).await.unwrap();
    u64::try_from(block.unwrap().timestamp).unwrap()
}

abigen!(IERC20, r#"[function decimals() external view returns (uint8)]"#);

#[allow(dead_code)]
pub async fn get_token_decimals(token_address: Address, provider: Arc<Provider<Http>>) -> AsyncResult<u8> {
    let token = IERC20::new(token_address, provider);
    let token_decimal = token.decimals().call().await.unwrap();
    Ok(token_decimal)
}

abigen!(UniswapV3Pool,
        r#"[{
            "constant": true,
            "inputs": [],
            "name": "token0",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [],
            "name": "token1",
            "outputs": [{"internalType": "address", "name": "", "type": "address"}],
            "stateMutability": "view",
            "type": "function"
        }]
       "#,);

#[allow(dead_code)]
pub async fn get_pool_tokens(pool_address: Address, provider: Arc<Provider<Http>>) -> AsyncResult<(Address, Address)> {
    let pool = UniswapV3Pool::new(pool_address, provider);
    let token0: Address = pool.token_0().call().await.unwrap();
    let token1: Address = pool.token_1().call().await.unwrap();
    Ok((token0, token1))
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::env;
    use std::sync::Arc;

    use chrono::{DateTime, Utc};
    use ethers::providers::{Http, Provider};
    use log::info;
    use util::async_result::AsyncResult;

    use crate::api_calls::*;
    use crate::subscription::tests::get_eth_subscription;

    fn get_eth_provider() -> Arc<Provider<Http>> {
        let alchemy_api_key = env::var("ALCHEMY_API_KEY").unwrap();
        let alchemy_url = format!("https://eth-mainnet.alchemyapi.io/v2/{}", alchemy_api_key);
        let provider = Provider::<Http>::try_from(alchemy_url).unwrap();
        Arc::new(provider)
    }

    async fn test_get_block_timestamp_async() -> AsyncResult<()> {
        let subscription = get_eth_subscription();
        let result = &subscription.params.as_ref().unwrap().result;
        let block_number = result.block_number();

        let provider = get_eth_provider();
        let timestamp = get_block_timestamp(block_number, provider).await;
        let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).expect("Invalid timestamp");
        info!("Block {} timestamp: {}", block_number, datetime.format("%Y-%m-%d %H:%M:%S UTC"));
        Ok(())
    }

    #[test]
    fn test_get_block_timestamp() {
        util_test::logging::init_logging();
        let _ = tokio_test::block_on(test_get_block_timestamp_async());
    }

    async fn test_get_token_decimals_async() -> AsyncResult<()> {
        let provider = get_eth_provider();

        let usdc_address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        let usdc_decimal = get_token_decimals(usdc_address.parse().unwrap(), provider.clone()).await.unwrap();
        assert_eq!(usdc_decimal, 6u8);

        let weth_address = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
        let weth_decimal = get_token_decimals(weth_address.parse().unwrap(), provider.clone()).await.unwrap();
        assert_eq!(weth_decimal, 18u8);
        Ok(())
    }

    #[test]
    fn test_get_token_decimals() {
        util_test::logging::init_logging();
        let _ = tokio_test::block_on(test_get_token_decimals_async());
    }

    async fn test_get_pool_tokens_async() -> AsyncResult<()> {
        let provider = get_eth_provider();
        let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"; // UDSC/WETH pool
        let (token0, token1) = get_pool_tokens(pool_address.parse().unwrap(), provider).await.unwrap();
        assert_eq!(token0, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse().unwrap()); // USDC : token0
        assert_eq!(token1, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".parse().unwrap()); // WETH : token1
        Ok(())
    }

    #[test]
    fn test_get_pool_tokens() {
        util_test::logging::init_logging();

        let _ = tokio_test::block_on(test_get_pool_tokens_async());
    }
}
