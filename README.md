<p align="center">
  <a href="https://github.com/axonweb3/axon">
    <img src="./docs/assets/logo/axon-01.png" width="450">
  </a>
  <p align="center">
	<a href="https://github.com/axonweb3/axon/releases"><img src="https://img.shields.io/github/v/release/axonweb3/axon?sort=semver"></a>
    <a href="https://github.com/axonweb3/axon/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg"></a>
    <a href="https://github.com/axonweb3/axon"><img src="https://github.com/axonweb3/axon/actions/workflows/web3_compatible.yml/badge.svg?branch=main"></a>
    <a href="https://github.com/axonweb3/axon"><img src="https://img.shields.io/github/contributors/axonweb3/axon"></a>
  </p>
  <p align="center">
     Developed by AxonWeb3<br>
  </p>
</p>

> Axon is still in active development.

## What is Axon

Axon is a high-performance layer2 framework with native cross-chain function. Built on top of the [Overlord](https://github.com/nervosnetwork/overlord) consensus protocol and the P2P network [Tentacle](https://github.com/nervosnetwork/tentacle)
, Axon supports hundreds of nodes and achieves thousands of TPS. Axon is also EVM-compatible with well-developed toolchains. Its high interoperability facilitates cross-chain communication among dApps.

## Highlights

### Developer-Friendly Design

Axon is compatible with [Ethereum](https://ethereum.org) so that all of the develop utilities can be used on it directly. And the [Axon CLI](https://github.com/axonweb3/axon-devops/axon-cli) provides an all-in-one client which includes initialization, DevOps, cross-chain request and so on. Seeing is believing, there is a [15 minutes tutorial]() that will lead you to build your own chain and deploy a tiny application.

### Native Cross-chain Communication

Openness and mobility are the foundation of social development, so is blockchain. Cross-chain function enhances liquidity for the web3 ecosystem. Axon develops native cross-chain communication without any bridge. Each the Axon-based chain can speak to [CKB](https://www.nervos.org), other Axon-based chains, and any [IBC](https://ibcprotocol.org) compatible chains. Axon will embed more cross-chain protocols in the future.

## Roadmap

Most of the infrastructure has been done and some substantial features to be developed are as below:

1. Compatible with [IBC](https://github.com/cosmos/ibc) protocol.
2. Implement the original homologous cross-chain protocol.
3. Design a cross-chain protocol for EVM-based chains.
4. Compatible with more cross-chain protocols in the future.

## Install

Axon provides the compiled binary on the [release page](`https://github.com/axonweb3/axon/release`), and if you want to build from source code, please make sure that [rust](https://www.rust-lang.org/), [clang](http://clang.org/), [openssl](https://www.openssl.org/), [m4](https://www.gnu.org/software/m4/) have already been installed. Then execute the following command:

```bash
# Clone from GitHub
git clone https://github.com/axonweb3/axon.git && cd axon
# Run release binary
cargo run --release -- -c devtools/chain/config.toml -g devtools/chain/genesis_single_node.json
```

## Toolchain

Apart from the framework, Axon has toolchains:

- [Axon Faucet](https://github.com/axonweb3/axon-faucet): the faucet for the Axon-based chains.
- [Axon Explorer](https://github.com/Magickbase/blockscan): a blockchain explorer for the Axon-based chains.
- [Axon DevOps](https://github.com/axonweb3/axon-devops): includes several utilities, such as monitor, benchmark tool and so on.

All the toolchains above can be dictated by **[Axon CLI](https://github.com/axonweb3/axon-cli), an exquisite and deft command line interface**. You can use Axon CLI to do anything related to Axon.

## Contributing

Please read the [CONTRIBUTING.md](./CONTRIBUTING.md) for details on code of conduct, and the process for submitting pull requests. And the security policy is described in [SECURITY.md](./SECURITY.md).

## Communication

Contact us through email: axon@axonweb3.io

Linktree: https://linktr.ee/axonweb3

## License

This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE) file for details.
