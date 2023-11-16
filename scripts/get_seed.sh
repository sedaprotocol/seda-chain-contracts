


#!/bin/bash

CHAIN_ID="sedachain"
DEV_ACCOUNT="seda10j9eatey68t072858rmunffygvtpwaz67se2gf"
RPC_URL="http://127.0.0.1:26657"
PROXY_CONTRACT_ADDRESS="seda1f6y3tfmmmpuu4a29gwr65dkwxrcymyavqsc2sk98rugrsj88dqaq457ug8"

OUTPUT="$(seda-chaind query wasm contract-state smart $PROXY_CONTRACT_ADDRESS '{"query_seed_request":{}}' --node $RPC_URL --output json --chain-id $CHAIN_ID)"
echo $OUTPUT















