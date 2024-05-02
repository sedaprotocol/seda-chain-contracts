#!/usr/bin/env bash

# Deploy the CosmWasm contract without creating a proposal
# This can only be used on devnet

# Exit if any command returns an error
set -eo pipefail

parent_path=$(cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
source ${parent_path}/../.env
source ${parent_path}/common.sh

# Check if seda binary is installed
check_binary
check_commands ${SEDA_BINARY_PATH} base64 uname jq

# Required variables
check_env_vars SEDA_CHAIN_RPC SEDA_CHAIN_ID SEDA_DEV_ACCOUNT

# Upload the proxy contract
proxy_contract_store_output="$($SEDA_BINARY_PATH tx wasm store ./artifacts/proxy_contract.wasm --node $SEDA_CHAIN_RPC --from $SEDA_DEV_ACCOUNT $TXN_GAS_FLAGS -y --output json --chain-id $SEDA_CHAIN_ID)"

echo $proxy_contract_store_output