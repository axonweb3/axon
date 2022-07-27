# Axon

> Axon is still in active development and a work in progress.

## About Axon

Axon is a sidechain framework based on [Muta](https://github.com/nervosnetwork/muta) with UDT token staking on CKB.
Dapp developers can use Axon toolkit to create and distribute chain-specific UDT on CKB. Chain token holders can stake UDT to become an Axon chain validator or just deposit UDT into Axon chain for use in Dapps.
Axon will support [interoperability 2.0](https://medium.com/nervosnetwork/blockchain-abstraction-and-interoperability-2-0-eea98d81b7b6) and be 100% Ethereum compatible, like [Godwoken](https://github.com/nervosnetwork/godwoken).

As a layer-2 solution on [Nervos](https://www.nervos.org/), Axon can achieve thousands of TPS with the help of [Overlord](https://github.com/nervosnetwork/overlord), a high-performance consensus. 
This offers Nervos DApp developers one more option. Godwoken is perfect for scenarios that involve low throughput and high value, such as DeFi, where security is usually more important than speed.
When it comes to consumer-facing applications such as decentralized games or social networks, Axon offers the best performance.

## Quick Start

Clone the source code, compile locally, and run the chain.
```shell
$ git clone https://github.com/nervosnetwork/axon.git
$ cd axon
$ cargo run --package axon --bin axon -- --config=./devtools/chain/config.toml --genesis=./devtools/chain/genesis_single_node.json
```
You can also use [axon-devops](https://github.com/nervosnetwork/axon-devops) to deploy a multi-node chain.

## EVM Testing
Download the required test files: 
```shell
$ git clone https://github.com/nervosnetwork/axon.git
$ cd axon
$ git submodule update --init --recursive  --depth=1
```
You can run tests with the following commands:
```shell
$ cp -r core/executor/res/ethereum/tests/BlockchainTests/GeneralStateTests/VMTests/vmTests/ core/executor/res/vmTests
$ cargo test --package core-executor --lib -- tests::vm_state_suite::test::run_tests --exact --nocapture
```

## Acknowledgment

Axon is based on [Muta](https://github.com/nervosnetwork/muta). Sincerely thanks to everyone who [contributed code to Muta](https://github.com/nervosnetwork/muta/graphs/contributors).
