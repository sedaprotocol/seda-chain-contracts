#!/bin/bash

CHAIN_ID="sedachain"
DEV_ACCOUNT="seda1qslt6t9fhmpl24azuxktesspfwkf6v9d2jpa5x"
RPC_URL="http://127.0.0.1:26657"
PROXY_CONTRACT_ADDRESS="seda14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9snnh0yy"

OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"register_data_request_executor":{"p2p_multi_address": "test_p2p_address" }}' --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID --amount 2seda)"
echo $OUTPUT