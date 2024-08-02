use std::convert::TryFrom;
use std::sync::Arc;

use alloy_primitives::U160;
use ethers::contract::abigen;
use ethers::prelude::{Address, Middleware};
use ethers::providers::{Http, Provider};
use util::prelude::AsyncResult;

pub(crate) async fn get_block_timestamp(block_number: u64, provider: Arc<Provider<Http>>) -> u64 {
    let block = provider.get_block(block_number).await.unwrap();
    u64::try_from(block.unwrap().timestamp).unwrap()
}

abigen!(IERC20, r#"[function decimals() external view returns (uint8)]"#);

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

pub async fn get_pool_tokens(pool_address: Address, provider: Arc<Provider<Http>>) -> AsyncResult<(Address, Address)> {
    let pool = UniswapV3Pool::new(pool_address, provider);
    let token0: Address = pool.token_0().call().await.unwrap();
    let token1: Address = pool.token_1().call().await.unwrap();
    Ok((token0, token1))
}

pub fn get_exchange_price(sqrt_price_x96: U160, token0_decimals: u8, token1_decimals: u8) -> f64 {
    let sqrt_price_x96 = u160_to_f64_lossy(sqrt_price_x96);
    let price = (sqrt_price_x96 * sqrt_price_x96) / (2.0_f64.powi(192));

    let token0_decimals = i32::from(token0_decimals);
    let token1_decimals = i32::from(token1_decimals);
    let exponent = token1_decimals - token0_decimals;
    10.0_f64.powi(exponent) / price
}

fn u160_to_f64_lossy(value: U160) -> f64 {
    let value_bytes = value.to_be_bytes_trimmed_vec();
    let u256_value = ethereum_types::U256::from_big_endian(&value_bytes);
    u256_to_f64_lossy(u256_value)
}

fn u256_to_f64_lossy(u256_value: ethereum_types::U256) -> f64 {
    // Reference: https://blog.m-ou.se/floats/
    // Step 1: Get leading zeroes
    let leading_zeroes = u256_value.leading_zeros();
    // Step 2: Get msb to be the farthest left bit
    let left_aligned = u256_value << leading_zeroes;
    // Step 3: Shift msb to fit in lower 53 bits of the first u64 (64-53=11)
    let quarter_aligned = left_aligned >> 11;
    let mantissa = quarter_aligned.0[3];
    // Step 4: For the dropped bits (all bits beyond the 53 most significant
    // We want to know only 2 things. If the msb of the dropped bits is 1 or 0,
    // and if any of the other bits are 1. (See blog for explanation)
    // So we take care to preserve the msb bit, while jumbling the rest of the bits
    // together so that any 1s will survive. If all 0s, then the result will also be 0.
    let dropped_bits = quarter_aligned.0[1] | quarter_aligned.0[0] | (left_aligned.0[0] & 0xFFFF_FFFF);
    let dropped_bits = (dropped_bits & 0x7FFF_FFFF_FFFF_FFFF) | (dropped_bits >> 63);
    let dropped_bits = quarter_aligned.0[2] | dropped_bits;
    // Step 5: dropped_bits contains the msb of the original bits and an OR-mixed 63 bits.
    // If msb of dropped bits is 0, it is mantissa + 0
    // If msb of dropped bits is 1, it is mantissa + 0 only if mantissa the lowest bit is 0
    // and other bits of the dropped bits are all 0 (which both can be tested with the below all at once)
    let mantissa = mantissa + ((dropped_bits - (dropped_bits >> 63 & !mantissa)) >> 63);
    // Step 6: Calculate the exponent
    // If self is 0, exponent should be 0 (special meaning) and mantissa will end up 0 too
    // Otherwise, (255 - n) + 1022, so it simplifies to 1277 - n
    // 1023 and 1022 are the cutoffs for the exponent having the msb next to the decimal point
    let exponent = if u256_value.is_zero() {
        0
    } else {
        1277 - leading_zeroes as u64
    };
    // Step 7: sign bit is always 0, exponent is shifted into place
    // Use addition instead of bitwise OR to saturate the exponent if mantissa overflows
    f64::from_bits((exponent << 52) + mantissa)
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
        let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"; // USDC/wETH pool
        let (token0, token1) = get_pool_tokens(pool_address.parse().unwrap(), provider).await.unwrap();
        assert_eq!(token0, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse().unwrap()); // USDC : token0
        assert_eq!(token1, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".parse().unwrap()); // wETH : token1
        Ok(())
    }

    #[test]
    fn test_get_pool_tokens() {
        util_test::logging::init_logging();
        let _ = tokio_test::block_on(test_get_pool_tokens_async());
    }

    #[test]
    fn test_get_exchange_price() {
        util_test::logging::init_logging();
        // see: https://blog.uniswap.org/uniswap-v3-math-primer
        let sqrt_price_x96 = U160::from(2018382873588440326581633304624437u128);
        let price = get_exchange_price(sqrt_price_x96, 6, 18);
        assert!((price - 1540.82f64).abs() < 0.01);
    }
}
