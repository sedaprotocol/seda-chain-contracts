#!/bin/bash




CHAIN_ID="sedachain"
DEV_ACCOUNT="seda10j9eatey68t072858rmunffygvtpwaz67se2gf"
RPC_URL="http://127.0.0.1:26657"


# Upload Proxy contract

OUTPUT="$(seda-chaind tx wasm store ./artifacts/proxy_contract.wasm --node $RPC_URL --from $DEV_ACCOUNT  --keyring-backend test --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
echo $OUTPUT

TXHASH=$(echo $OUTPUT | jq -r '.txhash')
echo $TXHASH

sleep 10

OUTPUT=$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)
echo $OUTPUT

PROXY_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo $PROXY_CODE_ID




# Instantiate Proxy

OUTPUT=$(seda-chaind tx wasm instantiate $PROXY_CODE_ID '{"token":"aseda"}' --no-admin --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --label proxy$PROXY_CODE_ID --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)
echo $OUTPUT

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

sleep 10


OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
echo $OUTPUT

PROXY_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "PROXY_CONTRACT_ADDRESS: " $PROXY_CONTRACT_ADDRESS



# Upload DataRequests contract

OUTPUT="$(seda-chaind tx wasm store artifacts/data_requests.wasm --node $RPC_URL --from $DEV_ACCOUNT  --keyring-backend test --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
echo $OUTPUT

TXHASH=$(echo $OUTPUT | jq -r '.txhash')

sleep 10


OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
echo $OUTPUT

DRs_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo $DRs_CODE_ID



# Instantiate DataRequests


OUTPUT=$(seda-chaind tx wasm instantiate $DRs_CODE_ID '{"token":"aseda", "proxy": "'$PROXY_CONTRACT_ADDRESS'" }' --no-admin --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --label dr$DRs_CODE_ID --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)
echo $OUTPUT

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

sleep 10


OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
echo $OUTPUT

DRs_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "DRs_CONTRACT_ADDRESS: " $DRs_CONTRACT_ADDRESS

  

# Upload Staking contract

OUTPUT="$(seda-chaind tx wasm store artifacts/staking.wasm --node $RPC_URL --from $DEV_ACCOUNT  --keyring-backend test --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
echo $OUTPUT

TXHASH=$(echo $OUTPUT | jq -r '.txhash')

sleep 10

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
echo $OUTPUT

STAKING_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo $STAKING_CODE_ID




# Instantiate Staking

OUTPUT=$(seda-chaind tx wasm instantiate $STAKING_CODE_ID '{"token":"aseda", "proxy":  "'$PROXY_CONTRACT_ADDRESS'" }' --no-admin --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --label staking$staking_code_id --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)
echo $OUTPUT

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

sleep 10


OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
echo $OUTPUT

STAKING_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "STAKING_CONTRACT_ADDRESS: " $STAKING_CONTRACT_ADDRESS



# Set DataRequests on Proxy

OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"set_data_requests":{"contract": "'$DRs_CONTRACT_ADDRESS'" }}' --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
echo $OUTPUT

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')
echo $TXHASH

sleep 10




# Set Staking on Proxy

OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"set_staking":{"contract": "'$STAKING_CONTRACT_ADDRESS'" }}' --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
echo $OUTPUT

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')
echo $TXHASH

echo "------------------------------------------\n"
echo "PROXY_CONTRACT_ADDRESS: " $PROXY_CONTRACT_ADDRESS
echo "DRs_CONTRACT_ADDRESS: " $DRs_CONTRACT_ADDRESS
echo "STAKING_CONTRACT_ADDRESS: " $STAKING_CONTRACT_ADDRESS
