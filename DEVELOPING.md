# Developing

[1]: https://rustup.rs/
[2]: https://github.com/WebAssembly/binaryen
[3]: https://github.com/sedaprotocol/seda-chain
[4]: https://nexte.st/


## Environment set up

- Install [rustup][1]. Once installed, make sure you have the wasm32 target:

  ```bash
  rustup default stable
  rustup update stable
  rustup target add wasm32-unknown-unknown
  ```

- Install [wasm-opt][2] (only if you need to produce an optimized version of the wasm): `cargo install wasm-opt --locked`

- Install [sedad][3]

## Compiling

You can build a release version, but not optimized with `cargo wasm` which outputs `target/wasm32-unknown-unknown/release/seda_contract.wasm`.

If you want an optimized version of the wasm you can instead run `cargo wasm-opt` which outputs to `target/seda_contract.wasm`. This is the version you would want to upload to the chain.

## Building Schema

You can build the json schema with `cargo schema`.

## Linting

`rustfmt` is used to format any Rust source code, we do use nightly format features: `cargo +nightly fmt`.

Nightly can be installed with: `rustup install nightly`. 

`clippy` is used as the linting tool: `cargo clippy -- -D warnings`

## Testing

### Unit

Unit testing can be done with: `cargo test`.

You could also install [nextest][4], with `cargo install cargo-nextest --locked`, then run `cargo nextest`. Nextest is a faster test runner for Rust.

### Fuzzing

Not yet set-up again.
<!-- 
To install fuzzing deps you can run:

```sh
make install-fuzz-deps
```

To list fuzz targets you can run:

```sh
make fuzz-list
```

> [!NOTE]
> The first time you do a `fuzz-run` command takes a very long time to build...
> This does cause the make command to timeout... not sure how to workaround that...

To run a fuzz target indefinitely:

```sh
FUZZ_TARGET=proxy-instantiate make fuzz-run
```

To run a fuzz target for a specifed amount of time:

```sh
TIME=1h FUZZ_TARGET=proxy-instantiate make fuzz-run-timeout
```

To re-run a found failing instance:

```sh
FUZZ_TARGET=proxy-instantiate ARTIFACT_PATH=./fuzz/artifacts/proxy-instantiate/crash-foo make fuzz-reproduce
```

To minify a found failing instance:

```sh
FUZZ_TARGET=proxy-instantiate ARTIFACT_PATH=./fuzz/artifacts/proxy-instantiate/crash-foo make fuzz-minify
```

When a failing instance is found the fuzzer will stop and tell you how to reproduce and mimize the test case:

Example output:

```bash
thread '<unnamed>' panicked at 'assertion failed: claimable_balance.amount > 0', fuzz_targets/fuzz_target_1.rs:130:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
==6102== ERROR: libFuzzer: deadly signal
    #0 0x561f6ae3a431  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x1c80431) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)
    #1 0x561f6e3855b0  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x51cb5b0) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)
    #2 0x561f6e35c08a  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x51a208a) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)
    #3 0x7fce05f5e08f  (/lib/x86_64-linux-gnu/libc.so.6+0x4308f) (BuildId: 1878e6b475720c7c51969e69ab2d276fae6d1dee)
    #4 0x7fce05f5e00a  (/lib/x86_64-linux-gnu/libc.so.6+0x4300a) (BuildId: 1878e6b475720c7c51969e69ab2d276fae6d1dee)
    #5 0x7fce05f3d858  (/lib/x86_64-linux-gnu/libc.so.6+0x22858) (BuildId: 1878e6b475720c7c51969e69ab2d276fae6d1dee)
    ...
    #27 0x561f6e3847b9  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x51ca7b9) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)
    #28 0x561f6ad98346  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x1bde346) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)
    #29 0x7fce05f3f082  (/lib/x86_64-linux-gnu/libc.so.6+0x24082) (BuildId: 1878e6b475720c7c51969e69ab2d276fae6d1dee)
    #30 0x561f6ad9837d  (/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x1bde37d) (BuildId: 6a95a932984a405ebab8171dddc9f812fdf16846)

NOTE: libFuzzer has rudimentary signal handlers.
      Combine libFuzzer with AddressSanitizer or similar for better crash reports.
SUMMARY: libFuzzer: deadly signal
MS: 0 ; base unit: 0000000000000000000000000000000000000000
0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x5d,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0x0,0xff,0x5f,0x5f,0x52,0xff,
\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000\000]\000\000\000\000\000\000\000\000\377__R\377
artifact_prefix='/home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/artifacts/fuzz_target_1/'; Test unit written to /home/azureuser/data/stellar/soroban-examples/fuzzing/fuzz/artifacts/fuzz_target_1/crash-04704b1542f61a21a4649e39023ec57ff502f627
Base64: AAAAAAAAAAAAAAAAAAAAAAAAXQAAAAAAAAAA/19fUv8=

────────────────────────────────────────────────────────────────────────────────

Failing input:

        fuzz/artifacts/fuzz_target_1/crash-04704b1542f61a21a4649e39023ec57ff502f627

Output of `std::fmt::Debug`:

        Input {
            deposit_amount: 0,
            claim_amount: -901525218878596739118967460911579136,
        }

Reproduce with:

        cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-04704b1542f61a21a4649e39023ec57ff502f627

Minimize test case with:

        cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-04704b1542f61a21a4649e39023ec57ff502f627

────────────────────────────────────────────────────────────────────────────────

Error: Fuzz target exited with exit status: 77
```

Just note the two following things:

1. To run cargo fuzz yourself currently on the this repo you must do `cargo +nightly-2024-01-21 fuzz ...`, or just run the commands above.
2. These failures are gitignored. The goal is to minimize and create a unit test. -->

## xtask

We use `cargo xtask` to help automate lot's of various actions.
It doesn't require any additional installations to use `xtask`, it's just a more rust-esque way of doing a `Makefile`.

You can read more about [xtask](https://github.com/matklad/cargo-xtask) and it's benefits at that link.