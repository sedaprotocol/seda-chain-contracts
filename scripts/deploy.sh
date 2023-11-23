#!/usr/bin/env bash

# usage: bash scripts/deploy.sh -c "sedachain" -d "seda12nmulx864e9ggymhf3tmavrmr6lse9l3qr4f4q" -r "http://127.0.0.1:26657"

source scripts/common.sh


# store_contract CONTRACT_NAME
store_contract(){

    OUTPUT="$(seda-chaind tx wasm store "./artifacts/$1.wasm" --node $RPC_URL --from $DEV_ACCOUNT  --keyring-backend test --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"
    echo $OUTPUT

    TXHASH=$(echo $OUTPUT | jq -r '.txhash')
    echo $TXHASH

    sleep 10

    OUTPUT=$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)
    echo $OUTPUT

    CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
  
}

# instantiate_contract CODE_ID INSTANTIATE_MSG LABEL
instantiate_contract() {

    OUTPUT=$(seda-chaind tx wasm instantiate $1 $2 --no-admin --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --label "$3$1" --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)
    echo $OUTPUT

    TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

    sleep 10


    OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"
    echo $OUTPUT

    CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')


}


# Upload Proxy contract
store_contract "proxy_contract"
PROXY_CODE_ID=$CODE_ID

# Instantiate Proxy
LABEL=proxy
INSTANTIATE_MSG='{"token":"aseda"}'
instantiate_contract $PROXY_CODE_ID $INSTANTIATE_MSG $LABEL
PROXY_CONTRACT_ADDRESS=$CONTRACT_ADDRESS

# Upload DataRequests contract
store_contract "data_requests"
DRs_CODE_ID=$CODE_ID

# Instantiate DataRequests
LABEL=drs
INSTANTIATE_MSG='{"token":"aseda","proxy":"'$PROXY_CONTRACT_ADDRESS'"}'
instantiate_contract $DRs_CODE_ID $INSTANTIATE_MSG $LABEL
DRs_CONTRACT_ADDRESS=$CONTRACT_ADDRESS

# Upload Staking contract
store_contract "staking"
STAKING_CODE_ID=$CODE_ID

# Instantiate Staking
LABEL=staking
INSTANTIATE_MSG='{"token":"aseda","proxy":"'$PROXY_CONTRACT_ADDRESS'"}'
instantiate_contract $STAKING_CODE_ID $INSTANTIATE_MSG $LABEL
STAKING_CONTRACT_ADDRESS=$CONTRACT_ADDRESS

# Set DataRequests on Proxy
EXECUTE_MSG='{"set_data_requests":{"contract":"'$DRs_CONTRACT_ADDRESS'"}}'
wasm_execute $EXECUTE_MSG 0

# Set Staking on Proxy
EXECUTE_MSG='{"set_staking":{"contract":"'$STAKING_CONTRACT_ADDRESS'"}}'
wasm_execute $EXECUTE_MSG 0

echo "------------------------------------------"
echo "PROXY_CODE_ID:" $PROXY_CODE_ID
echo "DRs_CODE_ID:" $DRs_CODE_ID
echo "STAKING_CODE_ID:" $STAKING_CODE_ID
echo "------------------------------------------"
echo "PROXY_CONTRACT_ADDRESS: " $PROXY_CONTRACT_ADDRESS
echo "DRs_CONTRACT_ADDRESS: " $DRs_CONTRACT_ADDRESS
echo "STAKING_CONTRACT_ADDRESS: " $STAKING_CONTRACT_ADDRESS
echo "------------------------------------------"



