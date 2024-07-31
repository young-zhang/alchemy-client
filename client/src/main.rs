mod swap_event;

use std::env;
extern crate log;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio_tungstenite::tungstenite::Message;
use util::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct EthereumSubscription {
    id: Option<i64>,
    #[serde(rename = "jsonrpc")]
    json_rpc: String,
    result: Option<String>,
    method: Option<String>,
    params: Option<Params>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    result: SubscriptionResult,
    subscription: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionResult {
    address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "transactionIndex")]
    transaction_index: String,
    #[serde(rename = "blockHash")]
    block_hash: String,
    #[serde(rename = "logIndex")]
    log_index: String,
    removed: bool,
}

fn parse_ethereum_subscription(json: &str) -> Result<EthereumSubscription, serde_json::Error> {
    serde_json::from_str(json)
}

fn parse_uniswap_v3(_subscription: &EthereumSubscription) {
    // TODO: Implement
}

pub async fn spawn_wss_client(endpoint: &str, json_rpc: &str) -> AsyncResult<()> {
    let endpoint = endpoint.to_string();
    let json_rpc = json_rpc.to_string();
    tokio::spawn(async move {
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
                                    println!("Received:");
                                    println!("{}", text);
                                    println!();
                                    let subscription = parse_ethereum_subscription(&text).unwrap();
                                    println!("{:#?}", subscription);
                                    println!();
                                    parse_uniswap_v3(&subscription);
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

    // let pool_address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"; // UDSC/ETH pool
    let pool_address = "0xCBCdF9626bC03E24f779434178A73a0B4bad62eD"; // wBTC/wETH pool

    let json_rpc = format!(r#"{{"jsonrpc":"2.0","method":"eth_subscribe","params":["logs",{{"address":"{}","topics":[]}}],"id":1}}"#,
                           pool_address);
    println!("json_rpc: {}", json_rpc);
    println!();

    spawn_wss_client(&endpoint, &json_rpc).await.unwrap();
    signal::ctrl_c().await?;
    println!("Received ctrl-c signal, shutting down...");

    Ok(())
}
