# Developing


## Environment set up

- Install [rustup][1]. Once installed, make sure you have the wasm32 target:

  ```bash
  rustup default stable
  rustup update stable
  rustup target add wasm32-unknown-unknown
  ```

- Install [Docker][2] (only if running cosmwasm/rust-optimizer image to compile contracts)

- Install [seda-chaind][3]

## Compiling and running tests

```sh
# this will produce wasm builds in ./target/wasm32-unknown-unknown/release
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACE=1 cargo unit-test

# auto-generate json schema
cargo schema -p data-requests
cargo schema -p proxy
cargo schema -p staking
```

## Linting

`rustfmt` is used to format any Rust source code:

```bash
cargo +nightly fmt
```

`clippy` is used as a linting tool:

```bash
cargo clippy
```

## Preparing the Wasm bytecode for production

Before we upload it to a chain, we need to ensure the smallest output size possible,
as this will be included in the body of a transaction. We also want to have a
reproducible build process, so third parties can verify that the uploaded Wasm
code did indeed come from the claimed Rust code.

The recommended method uses `rust-optimizer`, a docker image to
produce an extremely small build output in a consistent manner. The suggested way
to run it is this:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11
```

Or, If you're on an arm64 machine, you should use a docker image built with arm64.
```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.11
```

We must mount the contract code to `/code`. You can use a absolute path instead of `$(pwd)` if you don't want to `cd` to the directory first. The other two volumes are nice for speedup. Mounting `/code/target` in particular is useful to avoid docker overwriting your local dev files with root permissions. Note the `/code/target` cache is unique for each contract being compiled to limit interference, while the registry cache is global.

This is rather slow compared to local compilations, especially the first compile of a given contract. The use of the two volume caches is very useful to speed up following compiles of the same contract.

This produces an `artifacts` directory including a WASM binary for each contract, as well as `checksums.txt`, containing the Sha256 hash of the wasm file. The wasm file is compiled deterministically (anyone else running the same docker on the same git commit should get the identical file with the same Sha256 hash). It is also stripped and minimized for upload to a blockchain (we will also gzip it in the uploading process to make it even smaller).

## Verification

This project utilizes [`cargo-crev`](https://github.com/crev-dev/cargo-crev), a language and ecosystem agnostic, distributed code review system. For use, [see the Getting Started guide](https://github.com/crev-dev/cargo-crev/blob/master/cargo-crev/src/doc/getting_started.md).

Additionally, one may use [`cosmwasm-verify`](https://github.com/CosmWasm/cosmwasm-verify) to reproduce the build for verification. See the repository link for use.



## Uploading contracts and setting up cross-dependencies
To deploy and set up all contracts, the Proxy must first be instantiated followed by all of the other contracts, since the Proxy address is used as an argument when instantiating the other contracts. After all contracts are instantiated and to complete the circular dependency, the other contracts must then be set on the Proxy via Execute calls.

```bash
CHAIN_ID=seda-devnet | seda-testnet
```

Upload Proxy contract
```bash
OUTPUT="$(seda-chaind tx wasm store ./artifacts/proxy_contract.wasm --node $RPC_URL --from $DEV_ACCOUNT --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"

TXHASH=$(echo $OUTPUT | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"

PROXY_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
```


Instantiate Proxy
```bash
OUTPUT=$(seda-chaind tx wasm instantiate $PROXY_CODE_ID '{"token":"aseda"}' --no-admin --from $DEV_ACCOUNT --node $RPC_URL --label proxy$PROXY_CODE_ID --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"

PROXY_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
```

Upload DataRequests contract
```bash
OUTPUT="$(seda-chaind tx wasm store artifacts/data_requests.wasm --node $RPC_URL --from $DEV_ACCOUNT --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"

TXHASH=$(echo $OUTPUT | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $dr_store_tx_hash --node $RPC_URL --output json)"

DRs_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
```

Instantiate DataRequests

```bash
OUTPUT=$(seda-chaind tx wasm instantiate $DRs_CODE_ID '{"token":"aseda", "proxy": "'$PROXY_CONTRACT_ADDRESS'" }' --no-admin --from $DEV_ACCOUNT --node $RPC_URL --label dr$DRs_CODE_ID --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"

DRs_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
  ```

Upload Staking contract
```bash
OUTPUT="$(seda-chaind tx wasm store artifacts/staking.wasm --node $RPC_URL --from $DEV_ACCOUNT --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"

TXHASH=$(echo $OUTPUT | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"

STAKING_CODE_ID=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
```


Instantiate Staking
```bash
OUTPUT=$(seda-chaind tx wasm instantiate $STAKING_CODE_ID '{"token":"aseda", "proxy":  "'$PROXY_CONTRACT_ADDRESS'" }' --no-admin --from $DEV_ACCOUNT --node $RPC_URL --label staking$staking_code_id --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')

OUTPUT="$(seda-chaind query tx $TXHASH --node $RPC_URL --output json)"

STAKING_CONTRACT_ADDRESS=$(echo "$OUTPUT" | jq -r '.logs[].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
```

Set DataRequests on Proxy
```bash
OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"set_data_requests":{"contract": "'$DRs_CONTRACT_ADDRESS'" }}' --from $DEV_ACCOUNT --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')
```

Set Staking on Proxy
```bash
OUTPUT="$(seda-chaind tx wasm execute $PROXY_CONTRACT_ADDRESS '{"set_staking":{"contract": "'$STAKING_CONTRACT_ADDRESS'" }}' --from $DEV_ACCOUNT --node $RPC_URL --gas-prices 0.1aseda --gas auto --gas-adjustment 1.3 -y --output json --chain-id $CHAIN_ID)"

TXHASH=$(echo "$OUTPUT" | jq -r '.txhash')
```


## License
Contents of this repository are open source under [MIT License](LICENSE).

[1]: https://rustup.rs/
[2]: https://docs.docker.com/get-docker/
[3]: https://github.com/sedaprotocol/seda-chain