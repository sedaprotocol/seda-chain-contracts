#!/usr/bin/env bash

# If not defined, set to default values
TXN_GAS_FLAGS=${TXN_GAS_FLAGS:-"--gas-prices 100000000000aseda --gas auto --gas-adjustment 1.6"}
SLEEP_TIME=${SLEEP_TIME:-10}

# function: check if command exists
# Usage: check_commands <command_names...>
# Checks if the given commands are available in the system
check_commands(){
    local command_names=("$@")
    for command_name in "${command_names[@]}"; do
        if ! command -v ${command_name} > /dev/null 2>&1; then
            echo "Command \`${command_name}\` not found." >&2
            command_unset=true
        fi
    done
    [ -n "$command_unset" ] && exit 1

    return 0
}

# function: check if env vars are set
# Usage: check_env_vars <var_names...>
# Checks if the given environment variables are set
check_env_vars(){
    local var_names=("$@")
    for var_name in "${var_names[@]}"; do
        [ -z "${!var_name}" ] && echo "$var_name must be defined" >&2 && var_unset=true
    done
    [ -n "$var_unset" ] && exit 1

    return 0
}

# function: check if seda binary is installed
# Usage: check_binary
# Checks if the seda binary is installed and sets the path if found
check_binary(){
    if [ -z "${SEDA_BINARY_PATH}" ] && command -v sedad > /dev/null 2>&1; then
        SEDA_BINARY_PATH=$(command -v sedad) 
    else
        check_env_vars SEDA_BINARY_PATH
    fi
}


# function: update .env file <NAME> <VALUE>
# Usage: update_env_file <NAME> <VALUE>
# Updates the .env file with the provided name and value
update_env_file() {
    parent_path=$(cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
    echo "# $(date)" >> "${parent_path}/.env"
    echo "$1=$2" >> "${parent_path}/.env"
}

# function: store_contract CONTRACT_NAME_PATH
# Usage: store_contract <CONTRACT_NAME_PATH>
# Stores a contract using sedad
store_contract(){
    output="$(${SEDA_BINARY_PATH} tx wasm store "$1" --node $SEDA_CHAIN_RPC --from $SEDA_DEV_ACCOUNT --chain-id $SEDA_CHAIN_ID ${TXN_GAS_FLAGS} --output json -y)"
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
    echo "Waiting to query for CODE_ID..."
    sleep ${SLEEP_TIME}
    output=$(${SEDA_BINARY_PATH} query tx $txhash --node $SEDA_CHAIN_RPC --output json)
    CODE_ID=$(echo "$output" | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    echo "Deployed to CODE_ID=${CODE_ID}"
}

# function: instantiate_contract CODE_ID INSTANTIATE_MSG LABEL
# Usage: instantiate_contract <CODE_ID> <INSTANTIATE_MSG> <LABEL>
# Instantiates a contract using sedad
instantiate_contract(){
    output=$(${SEDA_BINARY_PATH} tx wasm instantiate $1 $2 --from $SEDA_DEV_ACCOUNT --admin $SEDA_DEV_ACCOUNT  --node $SEDA_CHAIN_RPC --label "$3" ${TXN_GAS_FLAGS} -y --output json --chain-id $SEDA_CHAIN_ID)
    txhash=$(echo "$output" | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
    echo "Waiting to query for CONTRACT_ADDRESS..."
    sleep ${SLEEP_TIME}
    output="$(${SEDA_BINARY_PATH} query tx $txhash --node $SEDA_CHAIN_RPC --output json)"
    CONTRACT_ADDRESS=$(echo "$output" | jq -r '.events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
    echo "Deployed to CONTRACT_ADDRESS=${CONTRACT_ADDRESS}"
}

# function: smart_query <QUERY_MSG>
# Usage: smart_query <QUERY_MSG>
# Queries the smart contract state using sedad
smart_query(){
    OUTPUT="$(seda-chaind query wasm contract-state smart $SEDA_CW_TARGET_CONTRACT $1 --node $SEDA_CHAIN_RPC --output json)"
    echo $OUTPUT
}

# function: wasm_execute <CONTRACT> <EXECUTE_MSG> <AMOUNT>
# Usage: wasm_execute <CONTRACT> <EXECUTE_MSG> <AMOUNT>
# Executes a wasm contract using sedad
wasm_execute(){
    command="${SEDA_BINARY_PATH} tx wasm execute $1 $2 --from $SEDA_DEV_ACCOUNT --node $SEDA_CHAIN_RPC ${TXN_GAS_FLAGS} -y --output json --chain-id $SEDA_CHAIN_ID --amount "$3"seda"
    echo "Command: $command"
    output="$($command)"
    echo $output
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
}

# # migrate_call <OLD_CONTRACT_ADDRESS> <NEW_CODE_ID> <MIGRATION_MSG>
# migrate_call(){
#     OUTPUT="$(${SEDA_BINARY_PATH} tx wasm migrate $1 $2 $3 --node $SEDA_CHAIN_RPC --output json --from $SEDA_DEV_ACCOUNT --node $SEDA_CHAIN_RPC ${TXN_GAS_FLAGS} -y --output json --chain-id $SEDA_CHAIN_ID)"
#     echo $OUTPUT
# }

# function: transfer_seda <FROM> <TO> <AMOUNT>
# Usage: transfer_seda <FROM> <TO> <AMOUNT>
# Transfers a specified amount of seda tokens from one account to another using sedad
transfer_seda(){
    command="${SEDA_BINARY_PATH} tx bank send $1 $2 ${3}seda --node $SEDA_CHAIN_RPC --chain-id $SEDA_CHAIN_ID ${TXN_GAS_FLAGS} --output json -y"
    echo "Command: $command"
    output="$($command)"
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
}

# function: submit proposal <JSON_FILE> <FROM>
# Usage: submit_proposal <JSON_FILE> <FROM>
# Submits a proposal using the specified JSON file and account using sedad
submit_proposal(){
    command="${SEDA_BINARY_PATH} tx group submit-proposal $1 --from $2 --node $SEDA_CHAIN_RPC --chain-id $SEDA_CHAIN_ID ${TXN_GAS_FLAGS} --output json -y"
    echo "Command: $command"
    output="$($command)"
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
}

# function: vote proposal <PROPOSAL ID> <FROM> <VOTE_OPTION> <METADATA>
# Usage: vote_proposal <PROPOSAL ID> <FROM> <VOTE_OPTION> <METADATA>
# Votes on a proposal with the specified ID, account, vote option, and metadata using sedad
vote_proposal(){
    command="${SEDA_BINARY_PATH} tx group vote $1 $2 $3 $4 --node $SEDA_CHAIN_RPC --chain-id $SEDA_CHAIN_ID ${TXN_GAS_FLAGS} --output json -y"
    echo "Command: $command"
    output="$($command)"
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
}

# function: execute proposal <PROPOSAL ID> <FROM>
# Usage: exec_proposal <PROPOSAL ID> <FROM>
# Executes a proposal with the specified ID and account using sedad
exec_proposal(){
    command="${SEDA_BINARY_PATH} tx group exec $1 --from $2 --node $SEDA_CHAIN_RPC --chain-id $SEDA_CHAIN_ID ${TXN_GAS_FLAGS} --output json -y"
    echo "Command: $command"
    output="$($command)"
    txhash=$(echo $output | jq -r '.txhash')
    echo "Transaction Hash: $txhash"
}

# function: query last proposal id
# Queries the ID of the last proposal using sedad
query_last_proposal_id(){
    command="${SEDA_BINARY_PATH} query group proposals-by-group-policy $SEDA_GROUP_POLICY --node $SEDA_CHAIN_RPC --output json"
    echo "Command: $command"
    proposal_id="$($command | jq '.proposals[-1].id' | tr -d '"')"
    echo "Last proposal ID: $proposal_id"
}

# function: query tally proposal <PROPOSAL_ID>
# Usage: query_tally_proposal <PROPOSAL_ID>
# Queries the tally result of a proposal with the specified ID using sedad
query_tally_proposal(){
    command="${SEDA_BINARY_PATH} query group tally-result $1 --node $SEDA_CHAIN_RPC --output json"
    echo "Command: $command"
    tally_result="$($command | jq)"
    echo "Tally: $tally_result"
}
