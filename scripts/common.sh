

while getopts ":c:d:r:p:" opt; do  
case $opt in
    c) CHAIN_ID=$OPTARG;;  
    d) DEV_ACCOUNT=$OPTARG;;  
    r) RPC_URL=$OPTARG;;  
    p) PROXY_CONTRACT_ADDRESS=$OPTARG;;  
    *) usage  
esac  
done  




# wasm_execute <EXECUTE_MSG> <AMOUNT>
wasm_execute() {

    OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS $1 --from $DEV_ACCOUNT  --keyring-backend test --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID --amount "$2"seda)"
    echo $OUTPUT

    TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')
    echo $TXHASH

    sleep 10
}


# wasm_query <QUERY_MSG>
wasm_query() {

    OUTPUT="$(seda-chaind query wasm contract-state smart $PROXY_CONTRACT_ADDRESS $1 --node $RPC_URL --output json)"
    echo $OUTPUT

    sleep 10
}
