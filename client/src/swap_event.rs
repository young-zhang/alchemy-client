use alloy_primitives::{FixedBytes, I256, I32, U128, U160};

// see: https://docs.uniswap.org/contracts/v3/reference/core/interfaces/pool/IUniswapV3PoolEvents
// int256,int256,uint160,uint128,int24
#[derive(Debug, PartialEq)]
pub struct SwapEvent {
   pub amount0: I256,        // int256
   pub amount1: I256,        // int256
   pub sqrt_price_x96: U160, // uint160
   pub liquidity: U128,      // uint128
   pub tick: I32,            // int24
}

#[allow(dead_code)]
pub fn parse_swap_event_from_data(data: &str) -> Result<SwapEvent, anyhow::Error> {
   let data = data.trim_start_matches("0x");

   let amount0_hex_str = &data[0..64];
   let amount1_hex_str = &data[64..128];
   let sqrt_price_x96_hex_str = &data[152..192];
   let liquidity_hex_str = &data[224..256];
   let tick_hex_str = &data[312..320];

   // println!("amount0: {}", amount0_hex_str);
   // println!("amount1: {}", amount1_hex_str);
   // println!("sqrt_price_x96: {}", sqrt_price_x96_hex_str);
   // println!("liquidity: {}", liquidity_hex_str);
   // println!("tick: {}", tick_hex_str);

   let amount0_bytes = amount0_hex_str.parse::<FixedBytes<32>>().unwrap();
   let amount1_bytes = amount1_hex_str.parse::<FixedBytes<32>>().unwrap();
   let sqrt_price_x96_bytes = sqrt_price_x96_hex_str.parse::<FixedBytes<20>>().unwrap();
   let liquidity_bytes = liquidity_hex_str.parse::<FixedBytes<16>>().unwrap();
   // even though this is only 3-bytes, we need to pad it to 4-bytes as we're parsing it to I32
   let tick_bytes = tick_hex_str.parse::<FixedBytes<4>>().unwrap();

   let amount0 = I256::from_be_bytes(amount0_bytes.0);
   let amount1 = I256::from_be_bytes(amount1_bytes.0);
   let sqrt_price_x96 = U160::from_be_bytes(sqrt_price_x96_bytes.0);
   let liquidity = U128::from_be_bytes(liquidity_bytes.0);
   let tick = I32::from_be_bytes(tick_bytes.0);

   Ok(SwapEvent { amount0,
                  amount1,
                  sqrt_price_x96,
                  liquidity,
                  tick })
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_parse_swap_event() {
      let data = "0xffffffffffffffffffffffffffffffffffffffffffffffffdacb2cb45d2d800000000000000000000000000000000000000000000000016edc8bc819f8b8b50900000000000000000000000000000000000000324b97f19a3936459e285115b700000000000000000000000000000000000000000000053f31c5c8cac5d23340000000000000000000000000000000000000000000000000000000000001321a";
      let data = data.trim_start_matches("0x");
      assert_eq!(data.len(), 32 * 5 * 2);
      let event = parse_swap_event_from_data(data).unwrap();

      assert_eq!(event.amount0, "-2681000000000000000".parse().unwrap());
      assert_eq!(event.amount1, "6767400346701675410697".parse().unwrap());
      assert_eq!(event.sqrt_price_x96, "3984803190183823086827191997879".parse().unwrap());
      assert_eq!(event.liquidity, "24777563784443426124608".parse().unwrap());
      assert_eq!(event.tick, "78362".parse().unwrap());
   }
}
