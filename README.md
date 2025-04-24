<p align="center">
  <a href="https://seda.xyz/">
    <img width="90%" alt="seda-chain-contracts" src="https://raw.githubusercontent.com/sedaprotocol/.github/refs/heads/main/images/banner.png">
  </a>
</p>

<h1 align="center">
  SEDA Chain Contracts
</h1>

<!-- The line below is for once the repo has CI to show build status. -->
<!-- [![Build Status][actions-badge]][actions-url] -->
[![GitHub Stars][github-stars-badge]](https://github.com/sedaprotocol/seda-chain-contracts)
[![GitHub Contributors][github-contributors-badge]](https://github.com/sedaprotocol/seda-chain-contracts/graphs/contributors)
[![Discord chat][discord-badge]][discord-url]
[![Twitter][twitter-badge]][twitter-url]

<!-- The line below is for once the repo has CI to show build status. -->
<!-- [actions-badge]: https://github.com/sedaprotocol/seda-chain-contracts/actions/workflows/push.yml/badge.svg -->
[actions-url]: https://github.com/sedaprotocol/seda-chain-contracts/actions/workflows/push.yml+branch%3Amain
[github-stars-badge]: https://img.shields.io/github/stars/sedaprotocol/seda-chain-contracts.svg?style=flat-square&label=github%20stars
[github-contributors-badge]: https://img.shields.io/github/contributors/sedaprotocol/seda-chain-contracts.svg?style=flat-square
[discord-badge]: https://img.shields.io/discord/500028886025895936.svg?logo=discord&style=flat-square
[discord-url]: https://discord.gg/seda
[twitter-badge]: https://img.shields.io/twitter/url/https/twitter.com/SedaProtocol.svg?style=social&label=Follow%20%40SedaProtocol
[twitter-url]: https://twitter.com/SedaProtocol

SEDA chain core contract written in CosmWasm.

To learn how to build a local version, please read [developing](DEVELOPING.md).
To learn how to contribute, please read [contributing](CONTRIBUTING.md).

The core contract enables some of the core features of the SEDA protocol. It allows:
- Users to post data requests.
- Users can also stake to become executors allowing them to commit & reveal on data requests.
- The chain, via sudo functions, to timeout & remove data requests, while rewarding/burning funds for the later.
- Owner to transfer ownership, change configurations, and management of the allowlist.

## Dependencies

Before starting, make sure you have [rustup](https://rustup.rs/) along with a recent `rustc` and `cargo` version installed. Currently, we are testing on 1.70.0+. You need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

Install [wasm-opt][https://github.com/WebAssembly/binaryen]: `cargo install wasm-opt --locked`, this produces a optimized version of the contract small enough to be uploaded to the chain.


## License

Contents of this repository are open source under [MIT License](LICENSE).
