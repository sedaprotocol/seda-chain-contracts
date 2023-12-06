#!/usr/bin/env bash

# usage: bash scripts/post_data_request.sh -c "sedachain" -d "seda12nmulx864e9ggymhf3tmavrmr6lse9l3qr4f4q" -r "http://127.0.0.1:26657" -p seda1e3cgamqcz46mnfwwwnwl98l4zwlux8pgjk4jgfa8l4hlz8cwlhksqqeh0g

source scripts/common.sh

wasm_execute '{"post_data_request":{"posted_dr":{"dr_id":[35,139,102,152,48,5,158,229,91,193,118,82,197,74,184,130,236,17,22,124,49,107,209,0,84,4,60,27,120,117,227,251],"dr_binary_id":[5,179,44,219,14,42,55,116,149,208,192,172,109,82,192,172,171,171,77,221,14,194,62,178,82,157,215,202,219,110,219,111],"tally_binary_id":[58,21,97,163,216,84,228,70,128,27,51,156,19,127,135,219,210,35,143,72,20,73,192,13,52,112,207,204,42,78,36,161],"dr_inputs":[],"tally_inputs":[],"memo":[141,39,233,7,88,225,128,39,63,248,148,102,94,181,92,159,162,115,148,82,189,90,247,10,95,54,31,107,184,115,201,229],"replication_factor":3,"gas_price":"10","gas_limit":"10","seda_payload":[],"payback_address":[]}}}' 2
