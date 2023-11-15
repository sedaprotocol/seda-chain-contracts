#!/bin/bash

CHAIN_ID="sedachain"
DEV_ACCOUNT="seda10j9eatey68t072858rmunffygvtpwaz67se2gf"
RPC_URL="http://127.0.0.1:26657"
PROXY_CONTRACT_ADDRESS="seda1q5pvpxv3f84pkxxzgv76e5ayvnkf8a46qze5rn8umng0gmyjspcs8d3ylr"

OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"register_data_request_executor":{"p2p_multi_address": "test_p2p_address" }}' --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID --amount 2seda)"
echo $OUTPUT