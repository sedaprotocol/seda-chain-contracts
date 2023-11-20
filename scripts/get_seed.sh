


#!/bin/bash

CHAIN_ID="sedachain"
DEV_ACCOUNT="seda1gt05eqfhk9acsxvzynafjackjvg26a2y08jllv"
RPC_URL="http://127.0.0.1:26657"
PROXY_CONTRACT_ADDRESS="seda14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9snnh0yy"

OUTPUT="$(seda-chaind query wasm contract-state smart $PROXY_CONTRACT_ADDRESS '{"query_seed_request":{}}' --node $RPC_URL --output json --chain-id $CHAIN_ID)"
echo $OUTPUT















