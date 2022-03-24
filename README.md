# Axon

> Axon is still under active development and considered to be a work in progress.

## About Axon

Axon is a sidechain framework based on [Muta](https://github.com/nervosnetwork/muta) with UDT token staking on CKB.
Dapp developers could use Axon toolkit to create and distribute chain-specific UDT on CKB, chain token holders could staking UDT to become an Axon chain validator, or just deposit it into Axon chain for use in dapps.
Axon will support [interoperability 2.0](https://medium.com/nervosnetwork/blockchain-abstraction-and-interoperability-2-0-eea98d81b7b6) and be 100% Ethereum compatible, like [Godwoken](https://github.com/nervosnetwork/godwoken).

As one of the layer-2 solutions of [Nervos](https://www.nervos.org/), Axon could achieve thousands of TPS with the help of the high-performance consensus, [Overlord](https://github.com/nervosnetwork/overlord). 
Thus just provides Nervos DApp developers with one more option. For low throughput high-value scenarios such as DeFi where security is usually a higher concern than speed, Godwoken is a perfect choice. 
For high throughput consumer-facing applications such as decentralized games or social networks, Axon could play the best.


## Quick Start

Clone the source code, locally compile and run a chain.
```shell
$ git clone https://github.com/nervosnetwork/axon.git
$ cd axon
$ cargo run --package axon --bin axon -- --config=./devtools/chain/config.toml --genesis=./devtools/chain/genesis_single_node.json
```
You can also refer to [axon-devops](https://github.com/nervosnetwork/axon-devops) to deploy multi-nodes chain.

## Acknowledgment

Axon is base on [Muta](https://github.com/nervosnetwork/muta). Sincerely thanks to everyone who [contributed code to Muta](https://github.com/nervosnetwork/muta/graphs/contributors).
