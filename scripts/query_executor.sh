#!/usr/bin/env bash

# usage: bash scripts/query_executor.sh -c "sedachain" -d "seda12nmulx864e9ggymhf3tmavrmr6lse9l3qr4f4q" -r "http://127.0.0.1:26657" -p seda1pwmxy357dhuy9hcnl0kdq0h89gkgls7zz4uswfwnl60f0f6fr2asgkclep


source scripts/common.sh

wasm_query '{"get_data_request_executor":{"executor":"'$DEV_ACCOUNT'"}}'










