# Axon

[![version](https://img.shields.io/github/v/release/nervosnetwork/axon?sort=semver)](https://github.com/nervosnetwork/axon/releases) [![license](https://img.shields.io/github/license/nervosnetwork/axon)](https://github.com/nervosnetwork/axon/blob/main/LICENSE) ![status-check](https://github.com/axonweb3/axon/actions/workflows/web3_compatible.yml/badge.svg?branch=main)  ![contributors](https://img.shields.io/github/contributors/nervosnetwork/axon)

> Axon is still in active development and a work in progress.

Axon is a high-performance layer 2 framework with native cross-chain function.  Built on top of the [Overlord](https://github.com/nervosnetwork/overlord) consensus protocol and the P2P network [Tentacle](https://github.com/nervosnetwork/tentacle), Axon supports hundreds of nodes and achieves thousands of TPS. Axon is also EVM-compatible with well-developed toolchains. It performs high interoperability, meaning deployed applications can achieve convenient cross-chain communication.


## Highlights

### About developer-friendly

Axon is compatible with [Ethereum](https://ethereum.org) so that all of the develop utilities can be used on it directly. And the [Axon CLI](https://github.com/axonweb3/axon-devops/axon-cli) provides an all-in-one client which includes initialization, DevOps, cross chain request and so on. If you consider seeing is believing, there is a [15 minutes tutorial]() that will lead you to build your own chain and deploy a tiny application.

### Native cross-chain communication

Openness and mobility are the foundation of social development, so is blockchain. Cross-chain derive the taking liquidity precisely. Axon satisfy native cross-chain ability without attach to a bridge. Each of the chain based Axon can cross to [CKB](https://www.nervos.org), any other Axon based chain, and [IBC](https://ibcprotocol.org) compatible chains. Axon will embed more cross-chain protocol in future.

## Roadmap

Most of the infrastructure has done and some substantial features to be developed are as below:

1. Be compatible with [IBC](https://github.com/cosmos/ibc) protocol.
2. Implement the original homologous cross chain protocol.
3. Implement some other cross chain protocol.

## Install

The [release page](`https://github.com/axonweb3/axon/release`) provides the compiled binary, and if you want to build from source code, please make sure that [rust](https://www.rust-lang.org/), [clang](http://clang.org/), [openssl](https://www.openssl.org/), [m4](https://www.gnu.org/software/m4/) have already been installed and then execute the following command:

```bash
git clone https://github.com/nervosnetwork/axon.git
cd axon && cargo build --release
```

## Toolchain

Apart from the framework, axon has corresponding toolchains:

- [Axon Faucet](https://github.com/axonweb3/axon-faucet): faucet for the chain based Axon.
- [Axon Explorer](): a blockchain explorer for the chain based Axon.
- [Axon DevOps](https://github.com/axonweb3/axon-devops): includes some utilities such as monitor, benchmark tool and so on.

All the toolchains above can be dictated by **[Axon CLI](https://github.com/axonweb3/axon-cli) which is a exquisite and deft command line interface**. You can use Axon CLI to do anything related Axon.

## Contributing

Please read the [CONTRIBUTING.md](./CONTRIBUTING.md) for details on code of conduct, and the process for submitting pull requests to us. And the security policy is described in [SECURITY.md](./SECURITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE.md](./LICENSE) file for details.

## Acknowledgment

Axon is based on [Muta](https://github.com/nervosnetwork/muta). Sincerely thanks to everyone who [contributed code to Muta](https://github.com/nervosnetwork/muta/graphs/contributors).
