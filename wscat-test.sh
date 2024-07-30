#!/bin/bash

if [ -z "$ALCHEMY_API_KEY" ]; then
    echo "ALCHEMY_API_KEY is not set!"
    exit 1
fi

# pool_ADDRESS='0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640' # USDC/ETH pool
POOL_ADDRESS='0xcbcdf9626bc03e24f779434178a73a0b4bad62ed' # wBTC/wETH pool

JSONRPC='{"jsonrpc":"2.0","method":"eth_subscribe","params":["logs",{"address":"'$POOL_ADDRESS'","topics":[]}],"id":1}'
echo "JSONRPC content:"
echo "$JSONRPC"
echo ""

wscat -c wss://eth-mainnet.g.alchemy.com/v2/$ALCHEMY_API_KEY -w 120 -x "$JSONRPC"
