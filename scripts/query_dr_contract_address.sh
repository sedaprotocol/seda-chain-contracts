


#!/bin/bash

CHAIN_ID="sedachain"
DEV_ACCOUNT="seda10j9eatey68t072858rmunffygvtpwaz67se2gf"
RPC_URL="http://127.0.0.1:26657"
PROXY_CONTRACT_ADDRESS="seda1q5pvpxv3f84pkxxzgv76e5ayvnkf8a46qze5rn8umng0gmyjspcs8d3ylr"

OUTPUT="$(seda-chaind query wasm contract-state smart $PROXY_CONTRACT_ADDRESS '{"get_data_requests_contract":{}}' --node $RPC_URL --output json --chain-id $CHAIN_ID)"
echo $OUTPUT















