mod api_calls;
mod subscription;
mod swap_event;

use std::convert::TryFrom;
use std::env;
use std::sync::Arc;

use chrono::{DateTime, Utc};

extern crate log;

use ethers::providers::{Http, Provider};
use futures_util::{SinkExt, StreamExt};
use subscription::{parse_ethereum_subscription, EthereumSubscription};
use tokio::signal;
use tokio_tungstenite::tungstenite::Message;
use util::prelude::*;

use crate::api_calls::get_block_timestamp;

async fn print_swap_details(subscription: &EthereumSubscription, provider: Arc<Provider<Http>>) -> AsyncResult<()> {
    if let Some(params) = &subscription.params {
        let block_number = &params.result.block_number();

        // we make a HTTP REST call to get the block timestamp - this is just for demonstration purposes
        let timestamp = get_block_timestamp(*block_number, provider).await;
        let timestamp = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).expect("Invalid timestamp");

        let data = &params.result.data;
        let swap_event = swap_event::parse_swap_event_from_data(data).unwrap();

        println!("timestamp: {} / block: {} / sqrtPriceX96: {}",
                 timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                 block_number,
                 swap_event.sqrt_price_x96);
    }
    Ok(())
}

pub async fn spawn_wss_client(endpoint: &str, json_rpc: &str) -> AsyncResult<()> {
    let endpoint = endpoint.to_string();
    let json_rpc = json_rpc.to_string();
    tokio::spawn(async move {
        let alchemy_api_key = env::var("ALCHEMY_API_KEY").unwrap();
        let alchemy_url = format!("https://eth-mainnet.alchemyapi.io/v2/{}", alchemy_api_key);
        let provider = Arc::new(Provider::<Http>::try_from(alchemy_url).unwrap());

        loop {
            match tokio_tungstenite::connect_async(&endpoint).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();

                    // Send JSON-RPC subscription
                    if let Err(e) = write.send(Message::Text(json_rpc.clone())).await {
                        eprintln!("Failed to send JSON-RPC: {}", e);
                        continue;
                    }

                    // Read incoming messages
                    while let Some(message) = read.next().await {
                        match message {
                            Ok(msg) => {
                                if let Message::Text(text) = msg {
                                    println!();
                                    // println!("Received:");
                                    // println!("{}", text);
                                    // println!();
                                    let subscription = parse_ethereum_subscription(&text).unwrap();
                                    // println!("{:#?}", subscription);
                                    // println!();
                                    print_swap_details(&subscription, provider.clone()).await.unwrap();
                                }
                            },
                            Err(e) => {
                                eprintln!("Error reading message: {}", e);
                                break;
                            },
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                },
            }
            eprintln!("WebSocket disconnected. Attempting to reconnect...");
        }
    });
    Ok(())
}

#[tokio::main]
#[allow(unused_must_use)]
async fn main() -> AsyncResult<()> {
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").map_err(|e| {
                                                         eprintln!("Couldn't read ALCHEMY_API_KEY: {}", e);
                                                         anyhow::anyhow!("Failed to read ALCHEMY_API_KEY")
                                                     })?;
    let endpoint = format!("wss://eth-mainnet.ws.alchemyapi.io/v2/{}", alchemy_api_key);
    println!("endpoint: {}", endpoint);

    // UDSC/ETH pool
    let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640";

    // wBTC/wETH pool
    // let pool_address = "0xCBCdF9626bC03E24f779434178A73a0B4bad62eD";

    // signature of Swap(address,address,int256,int256,uint160,uint128,int24)
    // see: https://www.4byte.directory/event-signatures/?bytes_signature=0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67
    let swap_signature = "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67";

    let json_rpc = format!(r#"{{"jsonrpc":"2.0","method":"eth_subscribe","params":["logs",{{"address":"{}","topics":["{}"]}}],"id":1}}"#,
                           pool_address, swap_signature);
    println!("json_rpc: {}", json_rpc);
    println!();

    spawn_wss_client(&endpoint, &json_rpc).await.unwrap();
    signal::ctrl_c().await?;
    println!("Received ctrl-c signal, shutting down...");

    Ok(())
}
