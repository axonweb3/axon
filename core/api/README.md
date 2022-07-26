# Axon JSON-RPC Protocols
This section lists the Axon JSON-RPC API endpoints. 

Axon JSON-RPC allow you to interact with a local or remote axon node using HTTP, IPC or WebSocket.

## JSONRPC Deprecation Process




## Table of Contents

* [RPC Methods](#rpc-methods)
    * [Gossip Methods](#Gossip-Methods)
        * [Method `eth_sendRawTransaction`](#method-eth_sendRawTransaction)
        * [Method `eth_blockNumber`](#method-eth_blockNumber)
		* [Method `eth_submitWork`](#method-eth_submitWork)
		* [Method `eth_submitHashrate`](#method-eth_submitHashrate)
		* [Method `pprof`](#method-pprof)
    * [History Methods](#History-Methods)
        * [Method `eth_getBlockByNumber`](#method-eth_getBlockByNumber)
        * [Method `eth_getBlockByHash`](#method-eth_getBlockByHash)
        * [Method `eth_getTransactionByHash`](#method-eth_getTransactionByHash)
		* [Method `eth_getBlockTransactionCountByNumber`](#method-eth_getBlockTransactionCountByNumber)
		* [Method `eth_getTransactionReceipt`](#method-eth_getTransactionReceipt)
		* [Method `eth_feeHistory`](#method-eth_feeHistory)
		* [Method `eth_getBlockTransactionCountByHash`](#method-eth_getBlockTransactionCountByHash)
		* [Method `eth_getTransactionByBlockHashAndIndex`](#method-eth_getTransactionByBlockHashAndIndex)
		* [Method `eth_getTransactionByBlockNumberAndIndex`](#method-eth_getTransactionByBlockNumberAndIndex)
		* [Method `getCrosschianResult`](#method-getCrosschianResult)
    * [State Methods](#State-Methods)
        * [Method `eth_getTransactionCount`](#method-eth_getTransactionCount)
        * [Method `eth_getBalance`](#method-eth_getBalance)
		* [Method `eth_chainId`](#method-eth_chainId)
		* [Method `net_version`](#method-net_version)
		* [Method `eth_call`](#method-eth_call)
		* [Method `eth_estimateGas`](#method-eth_estimateGas)
		* [Method `eth_getCode`](#method-eth_getCode)
		* [Method `eth_gasPrice`](#method-eth_gasPrice)
		* [Method `net_listening`](#method-net_listening)
		* [Method `eth_mining`](#method-eth_mining)
		* [Method `net_peerCount`](#method-net_peerCount)
		* [Method `eth_syncing`](#method-eth_syncing)
		* [Method `eth_getLogs`](#method-eth_getLogs)
		* [Method `web3_clientVersion`](#method-web3_clientVersion)
		* [Method `eth_accounts`](#method-eth_accounts)
		* [Method `web3_sha3`](#method-web3_sha3)
		* [Method `eth_getStorageAt`](#method-eth_getStorageAt)
		* [Method `eth_coinbase`](#method-eth_coinbase)
		* [Method `eth_hashrate`](#method-eth_hashrate)
* [RPC Errors](#rpc-errors)
* [RPC Types](#rpc-types)
    * [Type `BlockId`](#type-BlockId)
    * [Type `BlockView`](#type-BlockView)
    * [Type `H256`](#type-H256)
    * [Type `HeaderView`](#type-HeaderView)
    * [Type `TransactionView`](#type-TransactionView)


## RPC Methods

### Gossip-Methods

These methods track the head of the chain. This is how transactions make their way around the network, find their way into blocks, and how clients find out about new blocks.


#### Method `eth_sendRawTransaction`
* `eth_sendRawTransaction(data)`
    * `data`: [`Hex`](#type-hex) `|`
* result: [`H256`](#type-H256) `|` `zero hash`

Creates new message call transaction or a contract creation for signed transactions.

##### Params

*   `data` - 32 Bytes - the signed transactions hash.


##### Returns

DATA, 32 Bytes - the transaction hash, or the zero hash if the transaction is not yet available.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_sendRawTransaction",
  "params": [
    "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
  ]
}
```


Response


```
{
  "id": 1,
  "jsonrpc": "2.0",
   "result": "0xe670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331"
}
```

#### Method `eth_blockNumber`
* `eth_blockNumber()`
* result: [`U256`](#type-U256) `|` `zero`

Returns the number of most recent block.

##### Params

* None


##### Returns

Integer of the current block number the client is on

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_blockNumber",
  "params": [
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x3466",
	"id": 2
}
```

#### Method `eth_submitWork`
* `eth_submitWork(nc,hash,summary)`
    * `nc`: [`U256`](#type-U256)
    * `hash`: [`H256`](#type-H256)
    * `summary`: [`Hex`](#type-Hex)
* result: [`Boolean`](#type-Boolean) 

Used for submitting a proof-of-work solution.

##### Params

*   `nc` - 8 Bytes - The nonce found (64 bits)
*   `hash` -  32 Bytes - The header's pow-hash (256 bits)
*   `summary` - 32 Bytes - The mix digest (256 bits)
##### Returns

  Boolean - returns true if the provided solution is valid, otherwise false.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_submitWork",
	"params": [
	 "0x0000000000000001", 
	 "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", "0xD1GE5700000000000000000000000000D1GE5700000000000000000000000000"
	],
	"id": 64
}
```


Response


```
{
  "id":64,
  "jsonrpc":"2.0",
  "result": true
}

```

#### Method `eth_submitHashrate`
* `eth_submitHashrate(hash_rate,client_id)`
    * `hash_rate`: [`Hex`](#type-Hex)
    * `client_id`: [`Hex`](#type-Hex)
* result: [`Boolean`](#type-Boolean) 

Used for submitting mining hashrate.

##### Params

*   `hash_rate` - Hashrate, a hexadecimal string representation (32 bytes) of the hash rate
*   `client_id` -  ID, String - A random hexadecimal(32 bytes) ID identifying the client
##### Returns

  Boolean - returns true if submitting went through successfully and false otherwise.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_submitHashrate",
	"params": [
	 "0x0000000000000000000000000000000000000000000000000000000000500000", "0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c"
	],
	"id": 64
}
```


Response


```
{
  "id":64,
  "jsonrpc":"2.0",
  "result": true
}

```

#### Method `pprof`
* `pprof(enable)`
    * `enable`: [`Boolean`](#type-Boolean)
* result: [`Boolean`](#type-Boolean) 

TODO

##### Params

*   `enable` - TODO
##### Returns

  TODO

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "pprof",
	"params": [
	 true
	],
	"id": 64
}
```


Response


```
{
  "id":64,
  "jsonrpc":"2.0",
  "result": true
}

```

### History-Methods
Fetches historical records of every block back to genesis. This is like one large append-only file, and includes all block headers, block bodies, uncle blocks, and transaction receipts.

#### Method `eth_getBlockByHash`
* `eth_getBlockByHash(hash,show_rich_tx)`
    * `hash`: [`H256`](#type-H256) `|` 
    * `show_rich_tx`: [`Boolean`](#type-Boolean)
* result: [`BlockView`](#type-blockview) `|` `null`

Returns information about a block by hash.


##### Params

*   `block_hash` -  DATA, 32 Bytes - Hash of a block.
*   `show_rich_tx` -  Boolean, If true it returns the full transaction objects, if false only the hashes of the transactions.


##### Returns
The RPC returns the block details by block hash.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getBlockByHash",
  "params": [
    "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
    true
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"hash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"parentHash": "0xd619791daa02617ae2825a4ad7f2eb1379a069ac7b96b1628e75e1e654d5163c",
		"sha3Uncles": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"author": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"miner": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"stateRoot": "0xbd8e37758ba2c1d73f0c1dc3ea255f5b7c037f0e8e1ee9feb012461d58be236d",
		"transactionsRoot": "0x30fe4f21201c335b4501d517872d4d26bec39d350f987ee94e46e27bf7c48aae",
		"receiptsRoot": "0x77f8178b9f5a0e4aa59ee10d3d96cacfd0d6137fd3728cc01b5e5d6d74f6813f",
		"number": "0x1b4",
		"gasUsed": "0xc665442",
		"gasLimit": "0x1c9c380",
		"extraData": "0x",
		"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000",
		"timestamp": "0x62e00014",
		"difficulty": "0x1",
		"totalDifficulty": null,
		"sealFields": [],
		"baseFeePerGas": "0x539",
		"uncles": [],
		"transactions": [{
			"type": "0x2",
			"blockNumber": "0x1b4",
			"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
			"hash": "0xde3e1e12fb3f1a5b6ca32f580f0e7f170195f7a5233eb7d7095c53d7f5c519f5",
			"nonce": "0x2",
			"transactionIndex": "0x1b71",
			"from": "0x92df69a492c93d22c90247434b8d80944daa38fa",
			"to": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
			"value": "0x0",
			"gas": "0x0",
			"gasPrice": "0x77359400",
			"maxFeePerGas": "0x539",
			"maxPriorityFeePerGas": "0x77359400",
			"raw": "0x02f8af05028477359400847735940082ea6094ef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca80b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a07bf93b22eb0b40a85d0f8044d1b0201750ee1d8c27d558ae683509b13d8282e6a033e17c1e2b01fed2d742a37704530cd902754d726b96d41e24b5adc7be4dcdd3",
			"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
			"publicKey": "0x1113b46c2c52050fb9fd29cf88d4a9f8d253e17515dc142f72d8bb2cd18fdb3e45098e11811bcdd1487bafb7b24bf35c174cf6ede558059e23ad60d355f97070",
			"accessList": [],
			"chainId": "0x5",
			"v": "0x0",
			"r": "0x7bf93b22eb0b40a85d0f8044d1b0201750ee1d8c27d558ae683509b13d8282e6",
			"s": "0x33e17c1e2b01fed2d742a37704530cd902754d726b96d41e24b5adc7be4dcdd3"
		}],
		"size": "0x28f",
		"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"nonce": "0x0"
	},
	"id": 2
}

```


#### Method `eth_getTransactionByHash`
* `eth_getTransactionByHash(blockHash)`
    * `blockHash`: [`H256`](#type-H256)
* result: [`TransactionView`](#type-TransactionView) `|` `null`

Returns the information about a transaction requested by transaction hash.


##### Params

*   `blockHash` - Hash of a transaction.


##### Returns

Object - A transaction object, or null when no transaction was found:
- blockHash: DATA, 32 Bytes - hash of the block where this transaction was in. null when its pending.
- blockNumber: QUANTITY - block number where this transaction was in. null when its pending.
- from: DATA, 20 Bytes - address of the sender.
- gas: QUANTITY - gas provided by the sender.
- gasPrice: QUANTITY - gas price provided by the sender in Wei.
- hash: DATA, 32 Bytes - hash of the transaction.
- input: DATA - the data send along with the transaction. 
- nonce: QUANTITY - the number of transactions made by the sender prior to this one. 
- to: DATA, 20 Bytes - address of the receiver. null when its a contract creation transaction.
- transactionIndex: QUANTITY - integer of the transactions index position in the block. null when its pending.
- value: QUANTITY - value transferred in Wei.
- v: QUANTITY - ECDSA recovery id
- r: QUANTITY - ECDSA signature r
- s: QUANTITY - ECDSA signature s

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionByHash",
  "params": [
    "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"type": "0x2",
		"blockNumber": "0x1b4",
		"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"hash": "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65",
		"nonce": "0x1",
		"transactionIndex": "0x1b70",
		"from": "0x92df69a492c93d22c90247434b8d80944daa38fa",
		"to": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
		"value": "0x0",
		"gas": "0x73a9",
		"gasPrice": "0x77359400",
		"maxFeePerGas": "0x539",
		"maxPriorityFeePerGas": "0x77359400",
		"raw": "0x02f8af05018477359400847735940082ea6094ef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca80b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a0fa9bc76185d06c6e3178c66c40c195f374e80a78179392a84c1db731ce4d2d3da06079373330aa2c6d420267d83d8cd685db20638c7935f02a26fdf99dd010bfa2",
		"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
		"publicKey": "0x1113b46c2c52050fb9fd29cf88d4a9f8d253e17515dc142f72d8bb2cd18fdb3e45098e11811bcdd1487bafb7b24bf35c174cf6ede558059e23ad60d456798989",
		"accessList": [],
		"chainId": "0x5",
		"v": "0x0",
		"r": "0xfa9bc76185d06c6e3178c66c40c195f374e80a78179392a84c1db731ce4d2d3d",
		"s": "0x6079373330aa2c6d420267d83d8cd685db20638c7935f02a26fdf99dd010bfa2"
	},
	"id": 2
}

```

#### Method `eth_getBlockTransactionCountByNumber`
* `eth_getBlockTransactionCountByNumber(number)`
    * `number`: [`BlockId`](#type-BlockId)
* result: [`U256`](#type-U256) `|` `null`

Returns the number of transactions in a block matching the given block number.




##### Params

*   `bumber` - integer of a block number, or the string "earliest", "latest" or "pending".


##### Returns
Integer of the number of transactions in this block.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getBlockTransactionCountByNumber",
  "params": [
    "0xe90"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x1bb6",
	"id": 2
}

```

#### Method `eth_getTransactionReceipt`
* `eth_getTransactionReceipt(hash)`
    * `hash`: [`H256`](#type-H256)
* result: [`Web3Receipt`](#type-Web3Receipt) `|` `null`

Returns the receipt of a transaction by transaction hash.

*Note:* That the receipt is not available for pending transactions.


##### Params

*   `hash` - 32 Bytes - hash of a transaction.


##### Returns
* Object - A transaction receipt object, or null when no receipt was found.
	* transactionHash : DATA, 32 Bytes - hash of the transaction.
	* transactionIndex: QUANTITY - integer of the transactions index position in the block.
	* blockHash: DATA, 32 Bytes - hash of the block where this transaction was in.
	* blockNumber: QUANTITY - block number where this transaction was in.
	* from: DATA, 20 Bytes - address of the sender.
	* to: DATA, 20 Bytes - address of the receiver. null when its a contract creation transaction.
	* cumulativeGasUsed : QUANTITY - The total amount of gas used when this transaction was executed in the block.
	* gasUsed : QUANTITY - The amount of gas used by this specific transaction alone.
	* contractAddress : DATA, 20 Bytes - The contract address created, if the transaction was a * contract creation, otherwise null.
	* logs: Array - Array of log objects, which this transaction generated.
	* logsBloom: DATA, 256 Bytes - Bloom filter for light clients to quickly retrieve related logs. It also returns either :
	* root : DATA 32 bytes of post-transaction stateroot (pre Byzantium)
	* status: QUANTITY either 1 (success) or 0 (failure)

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionReceipt",
  "params": [
    "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"blockNumber": "0x1b4",
		"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"contractAddress": null,
		"cumulativeGasUsed": "0x73a9",
		"effectiveGasPrice": "0x73a9",
		"from": "0x92df69a492c93d22c90247434b8d80944daa38fa",
		"gasUsed": "0x73a9",
		"logs": [{
			"address": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
			"topics": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef", "0x00000000000000000000000092df69a492c93d22c90247434b8d80944daa38fa", "0x0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e8915"],
			"data": "0x0000000000000000000000000000000000000000000000000000000000000000",
			"blockNumber": "0x1b4",
			"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
			"transactionHash": "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65",
			"transactionIndex": "0x1b70",
			"logIndex": "0x0",
			"removed": false
		}],
		"logsBloom": "0x20000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000002008000000000000000000000000000000002000000000000000008000000000000000000040000000004000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000",
		"root": "0xbd8e37758ba2c1d73f0c1dc3ea255f5b7c037f0e8e1ee9feb012461d58be236d",
		"status": "0x1",
		"to": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
		"transactionHash": "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65",
		"transactionIndex": "0x1b70",
		"type": "0x2"
	},
	"id": 2
}

```

#### Method `eth_feeHistory`
* `eth_feeHistory(block_count,newest_block,reward_percentiles)`
    * `block_count`: [`U256`](#type-U256)
	* `newest_block`: [`BlockId`](#type-BlockId)
	* `reward_percentiles`: `Array<` [`f64`](#type-f64) `>`
* result: [`Web3FeeHistory`](#type-Web3FeeHistory)

TODO


##### Params

*   `block_count` - 256-bit unsigned integer.
*   `newest_block` - integer of a block number, or the string "earliest", "latest" or "pending".
*   `reward_percentiles` - TODO


##### Returns
* TODO

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionReceipt",
  "params": [
    "0x8",
	"0x1b4"
  ]
}
```


Response


```
TODO
{
	"jsonrpc": "2.0",
	"result": {
		"oldestBlock": "0x0",
		"reward": null,
		"baseFeePerGas": [],
		"gasUsedRatio": []
	},
	"id": 2
}

```

#### Method `eth_getBlockTransactionCountByHash`
* `eth_getBlockTransactionCountByHash(hash)`
    * `hash`: [`Hash`](#type-Hash)
* result: [`U256`](#type-u256) 

Returns the number of transactions in a block from a block matching the given block hash.


##### Params

*   `hash` - 32 Bytes - hash of a block



##### Returns

Integer of the number of transactions in this block.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getBlockTransactionCountByHash",
  "params": [
    "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x1b72",
	"id": 2
}

```

#### Method `eth_getTransactionByBlockHashAndIndex`
* `eth_getTransactionByBlockHashAndIndex(hash,position)`
    * `hash`: [`Hash`](#type-Hash)
	* `position`: [`U256`](#type-U256)
* result: [`TransactionView`](#type-TransactionView) 

Returns information about a transaction by block hash and transaction index position.


##### Params

*   `hash` - 32 Bytes - hash of a block
*   `position` - integer of the transaction index position.



##### Returns

* Object - A transaction object, or null when no transaction was found:
	* blockHash: DATA, 32 Bytes - hash of the block where this transaction was in. null when its pending.
	* blockNumber: QUANTITY - block number where this transaction was in. null when its pending.
	* from: DATA, 20 Bytes - address of the sender.
	* gas: QUANTITY - gas provided by the sender.
	* gasPrice: QUANTITY - gas price provided by the sender in Wei.
	* hash: DATA, 32 Bytes - hash of the transaction.
	* input: DATA - the data send along with the transaction.
	* nonce: QUANTITY - the number of transactions made by the sender prior to this one.
	* to: DATA, 20 Bytes - address of the receiver. null when its a contract creation transaction.
	* transactionIndex: QUANTITY - integer of the transactions index position in the block. null when its pending.
	* value: QUANTITY - value transferred in Wei.
	* v: QUANTITY - ECDSA recovery id
	* r: QUANTITY - ECDSA signature r
	* s: QUANTITY - ECDSA signature s


##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionByBlockHashAndIndex",
  "params": [
     "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65",
     "0x8"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"type": "0x2",
		"blockNumber": "0x1b4",
		"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"hash": "0x0f7c17ba06dcb8106dd6956bfdb131cd48cfeaf2dfa21b4f16384d0a872441a2",
		"nonce": "0x8",
		"transactionIndex": "0x8",
		"from": "0xae32a32bdad774608ab473d7ca1993e9921b6a7a",
		"to": "0x057ef64e23666f000b34ae31332854acbd1c8544",
		"value": "0x0",
		"gas": "0x73a9",
		"gasPrice": "0x77359400",
		"maxFeePerGas": "0x539",
		"maxPriorityFeePerGas": "0x77359400",
		"raw": "0x02f8af05088477359400847735940082ea6094057ef64e23666f000b34ae31332854acbd1c854480b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a0bd71f0ff2689b002dd58512713d91321266f23dcb0900e2b20f33e2be2222627a051f87230181b70d4736f907defbe505b2a3048e50113881c2254814c3f4ccb47",
		"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
		"publicKey": "0x5651add842f3f79235ca3958f6fafa300d9c54b0632d9bbfb5d8eb926caabdf08a78e3492537223efb295324aae4080348c86bffaa4235867f10766f12345678",
		"accessList": [],
		"chainId": "0x5",
		"v": "0x0",
		"r": "0xbd71f0ff2689b002dd58512713d91321266f23dcb0900e2b20f33e2be2222627",
		"s": "0x51f87230181b70d4736f907defbe505b2a3048e50113881c2254814c3f4ccb47"
	},
	"id": 2
}

```

#### Method `eth_getTransactionByBlockNumberAndIndex`
* `eth_getTransactionByBlockNumberAndIndex(number,position)`
    * `number`: [`BlockId`](#type-BlockId)
	* `position`: [`U256`](#type-U256)
* result: [`TransactionView`](#type-TransactionView) 

Returns information about a transaction by block number and transaction index position.


##### Params

*   `number` -  a block number, or the string "earliest", "latest" or "pending"
*   `position` - integer of the transaction index position.



##### Returns

* Object - A transaction object, or null when no transaction was found:
	* blockHash: DATA, 32 Bytes - hash of the block where this transaction was in. null when its pending.
	* blockNumber: QUANTITY - block number where this transaction was in. null when its pending.
	* from: DATA, 20 Bytes - address of the sender.
	* gas: QUANTITY - gas provided by the sender.
	* gasPrice: QUANTITY - gas price provided by the sender in Wei.
	* hash: DATA, 32 Bytes - hash of the transaction.
	* input: DATA - the data send along with the transaction.
	* nonce: QUANTITY - the number of transactions made by the sender prior to this one.
	* to: DATA, 20 Bytes - address of the receiver. null when its a contract creation transaction.
	* transactionIndex: QUANTITY - integer of the transactions index position in the block. null when its pending.
	* value: QUANTITY - value transferred in Wei.
	* v: QUANTITY - ECDSA recovery id
	* r: QUANTITY - ECDSA signature r
	* s: QUANTITY - ECDSA signature s


##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionByBlockNumberAndIndex",
  "params": [
     "0xb14",
     "0x8"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"type": "0x2",
		"blockNumber": "0xb14",
		"blockHash": "0xcaca33a66b32c99d157af1a9f1940c878e37dab6a95c0496a44852c91218658a",
		"hash": "0x401ee5194c6628800c3437618fe47645d065fae5dcf43c540ebe7d76aa7d4be0",
		"nonce": "0x123",
		"transactionIndex": "0x8",
		"from": "0x12247217ada7661c30a92425b62c756f54bfb5fc",
		"to": "0x5fbdb2315678afecb367f032d93f642f64180aa3",
		"value": "0x0",
		"gas": "0x73a9",
		"gasPrice": "0x77359400",
		"maxFeePerGas": "0x539",
		"maxPriorityFeePerGas": "0x77359400",
		"raw": "0x02f8b1058201238477359400847735940082ea60945fbdb2315678afecb367f032d93f642f64180aa380b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a0df97bdb5395f8a6a78133ea75b2dfa1fe0edd54710efe1af5766478b472b2719a036d810b450963968711e03c7b7032a1ca9d4cd5f507dbcd6a5299ab46042871d",
		"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
		"publicKey": "0x5d29609b19fc299039853e319d7be4a153c4c464beca159bef943bef4d50c6fc1180058d25a56d824a32725040d394bb4251c935e834ae2bc7174361fff4dc86",
		"accessList": [],
		"chainId": "0x5",
		"v": "0x0",
		"r": "0xdf97bdb5395f8a6a78133ea75b2dfa1fe0edd54710efe1af5766478b472b2719",
		"s": "0x36d810b450963968711e03c7b7032a1ca9d4cd5f507dbcd6a5299ab46042871d"
	},
	"id": 2
}

```

#### Method `getCrosschianResult`
* `getCrosschianResult(tx_hash)`
    * `tx_hash`: [`H256`](#type-H256)
* result: [`CrossChainTransaction`](#type-CrossChainTransaction) `|` `null`

TODO


##### Params

*   `tx_hash` - Hash of transaction



##### Returns

* Object - A transaction object, or null when no transaction was found:
	


##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "getCrosschianResult",
  "params": [
     "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": null,
	"id": 2
}

```

#### Method `eth_getBlockByNumber`
* `eth_getBlockByNumber(number,show_rich_tx)`
    * `number`: [`BlockId`](#type-BlockId) `|` `"earliest"``|` `"latest"``|` `"pending"`
    * `show_rich_tx`: [`Boolean`](#type-Boolean)
* result: [`BlockView`](#type-blockview) `|` `null`

Returns information about a block by block number.


##### Params

*   `number` -  integer of a block number, or the string "earliest", "latest" or "pending"
*   `show_rich_tx` -  Boolean,If true it returns the full transaction objects, if false only the hashes of the transactions.


##### Returns
The RPC returns the block details.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getBlockByNumber",
  "params": [
    "0x1b4",
    true
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"hash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"parentHash": "0xd619791daa02617ae2825a4ad7f2eb1379a069ac7b96b1628e75e1e654d5163c",
		"sha3Uncles": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"author": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"miner": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"stateRoot": "0xbd8e37758ba2c1d73f0c1dc3ea255f5b7c037f0e8e1ee9feb012461d58be236d",
		"transactionsRoot": "0x30fe4f21201c335b4501d517872d4d26bec39d350f987ee94e46e27bf7c48aae",
		"receiptsRoot": "0x77f8178b9f5a0e4aa59ee10d3d96cacfd0d6137fd3728cc01b5e5d6d74f6813f",
		"number": "0x1b4",
		"gasUsed": "0xc665442",
		"gasLimit": "0x1c9c380",
		"extraData": "0x",
		"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000",
		"timestamp": "0x62e00014",
		"difficulty": "0x1",
		"totalDifficulty": null,
		"sealFields": [],
		"baseFeePerGas": "0x539",
		"uncles": [],
		"transactions": [{
			"type": "0x2",
			"blockNumber": "0x1b4",
			"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
			"hash": "0xde3e1e12fb3f1a5b6ca32f580f0e7f170195f7a5233eb7d7095c53d7f5c519f5",
			"nonce": "0x2",
			"transactionIndex": "0x1b71",
			"from": "0x92df69a492c93d22c90247434b8d80944daa38fa",
			"to": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
			"value": "0x0",
			"gas": "0x0",
			"gasPrice": "0x77359400",
			"maxFeePerGas": "0x539",
			"maxPriorityFeePerGas": "0x77359400",
			"raw": "0x02f8af05028477359400847735940082ea6094ef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca80b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a07bf93b22eb0b40a85d0f8044d1b0201750ee1d8c27d558ae683509b13d8282e6a033e17c1e2b01fed2d742a37704530cd902754d726b96d41e24b5adc7be4dcdd3",
			"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
			"publicKey": "0x1113b46c2c52050fb9fd29cf88d4a9f8d253e17515dc142f72d8bb2cd18fdb3e45098e11811bcdd1487bafb7b24bf35c174cf6ede558059e23ad60d355f97070",
			"accessList": [],
			"chainId": "0x5",
			"v": "0x0",
			"r": "0x7bf93b22eb0b40a85d0f8044d1b0201750ee1d8c27d558ae683509b13d8282e6",
			"s": "0x33e17c1e2b01fed2d742a37704530cd902754d726b96d41e24b5adc7be4dcdd3"
		}],
		"size": "0x28f",
		"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"nonce": "0x0"
	},
	"id": 2
}

```

### State-Methods
Methods that report the current state of all the data stored. The "state" is like one big shared piece of RAM, and includes account balances, contract data, and gas estimations.

#### Method `eth_getTransactionCount`
* `eth_getTransactionCount(address,number)`
    * `address`: [`H160`](#type-H160)
    * `number`: [`BlockId`](#type-BlockId)`|` `null`
* result: [`U256`](#type-U256) `|` `null`

Returns the number of transactions sent from an address.


##### Params

*   `address` - 20 Bytes - address.
*   `number` - integer block number, or the string "latest", "earliest" or "pending"

##### Returns

 integer of the number of transactions send from this address.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getTransactionCount",
  "params": [
    "0x92df69a492c93d22c90247434b8d80944daa38fa"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x12c",
	"id": 2
}

```

#### Method `eth_getBalance`
* `eth_getBalance(address,number)`
    * `address`: [`H160`](#type-H160)
    * `number`: [`BlockId`](#type-BlockId)`|` `null`
* result: [`U256`](#type-U256) `|` `null`

Returns the balance of the account of given address.


##### Params

*   `address` - 20 Bytes - address to check for balance.
*   `block_number` - integer block number, or the string "latest", "earliest" or "pending"

##### Returns

Integer of the current balance in wei.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getBalance",
  "params": [
    "0x92df69a492c93d22c90247434b8d80944daa38fa"
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x8459515c8443cde72e000",
	"id": 2
}

```

#### Method `eth_chainId`
* `eth_chainId()`
* result: [`U256`](#type-U256) `|` `null`

Returns the chain_id of axon network.


##### Params

*  None

##### Returns

Current chain id.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_chainId",
  "params": [
   
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x5",
	"id": 2
}

```
#### Method `net_version`
* `net_version()`
* result: [`U256`](#type-U256) `|` `null`

Returns the current network id.


##### Params

*  None

##### Returns

String - The current network id.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "net_version",
  "params": [
   
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x5",
	"id": 2
}

```
#### Method `eth_call`
* `eth_call(req,number)`
    * `req`: [`Web3CallRequest`](#type-Web3CallRequest)
	* `number`: [`BlockId`](#type-BlockId)
* result: [`Hex`](#type-Hex) `|` `null`

Executes a new message call immediately without creating a transaction on the block chain.


##### Params

* `req` - The transaction call object

	* from: DATA, 20 Bytes - (optional) The address the transaction is sent from.
	* to: DATA, 20 Bytes - The address the transaction is directed to.
	* gas: QUANTITY - (optional) Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.
	* gasPrice: QUANTITY - (optional) Integer of the gasPrice used for each paid gas
	* value: QUANTITY - (optional) Integer of the value sent with this transaction
	* data: DATA - (optional) Hash of the method signature and encoded parameters.
*   `block_number` - integer block number, or the string "latest", "earliest" or "pending"

##### Returns

 DATA - the return value of executed contract.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_call",
	"params": [{
		see above
	}],
	"id": 1
}
```


Response


```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": "0x"
}

```
#### Method `eth_estimateGas`
* `eth_estimateGas(req,number)`
    * `req`: [`Web3CallRequest`](#type-Web3CallRequest)
	* `number`: [`BlockId`](#type-BlockId)
* result: [`U256`](#type-U256) `|` `null`

Generates and returns an estimate of how much gas is necessary to allow the transaction to complete. The transaction will not be added to the blockchain. Note that the estimate may be significantly more than the amount of gas actually used by the transaction, for a variety of reasons including EVM mechanics and node performance.


##### Params

* `req` - The transaction call object. Expect all properties are optional. If no gas limit is specified geth uses the block gas limit from the pending block as an upper bound. As a result the returned estimate might not be enough to executed the call/transaction when the amount of gas is higher than the pending block gas limit.

	* from: DATA, 20 Bytes - (optional) The address the transaction is sent from.
	* to: DATA, 20 Bytes - The address the transaction is directed to.
	* gas: QUANTITY - (optional) Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.
	* gasPrice: QUANTITY - (optional) Integer of the gasPrice used for each paid gas
	* value: QUANTITY - (optional) Integer of the value sent with this transaction
	* data: DATA - (optional) Hash of the method signature and encoded parameters.
* `number` - integer block number, or the string "latest", "earliest" or "pending"

##### Returns

 The amount of gas used.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_call",
	"params": [{
		see above
	}],
	"id": 1
}
```


Response


```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": "0x5208" // 21000
}

```
#### Method `eth_getCode`
* `eth_getCode(address,block_number)`
    * `address`: [`H160`](#type-H160)
	* `block_number`: [`BlockId`](#type-BlockId)
* result: [`Hex`](#type-Hex) `|` `null`

Returns code at a given address.


##### Params

* `address` - DATA, 20 Bytes - address.
* `block_number` - integer block number, or the string "latest", "earliest" or "pending"

##### Returns

 The code from the given address.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_getCode",
	"params": [{
	"0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b",
    "0x2",
	}],
	"id": 1
}
```


Response


```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": "0x600160008035811a818181146012578301005b601b6001356025565b8060005260206000f25b600060078202905091905056"
}

```

#### Method `eth_gasPrice`
* `eth_gasPrice()`
    
* result: [`U256`](#type-U256) `|`

Returns the current price per gas in wei.


##### Params

* None

##### Returns

 Integer of the current gas price in wei.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_gasPrice",
	"params": [],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x8",
	"id": 2
}
```

#### Method `net_listening`
* `net_listening()`
    
* result: [`Boolean`](#type-Boolean) `|`

Returns true if client is actively listening for network connections.


##### Params

* None

##### Returns

 Boolean - true when listening, otherwise false.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "net_listening",
	"params": [],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": true,
	"id": 2
}
```
#### Method `eth_mining`
* `eth_mining()`
    
* result: [`Boolean`](#type-Boolean) `|`

Returns true if client is actively mining new blocks.


##### Params

* None

##### Returns

 Boolean - returns true of the client is mining, otherwise false.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_mining",
	"params": [],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": false,
	"id": 2
}
```

#### Method `net_peerCount`
* `net_peerCount()`
    
* result: [`U256`](#type-U256) `|`

Returns number of peers currently connected to the client.


##### Params

* None

##### Returns

  Integer of the number of connected peers.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "net_peerCount",
	"params": [],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x7",
	"id": 2
}
```

#### Method `eth_syncing`
* `eth_syncing()`
    
* result: [`Web3SyncStatus`](#type-Web3SyncStatus) `|` `false`

Returns an object with data about the sync status or false.


##### Params

* None

##### Returns

* Object|Boolean, An object with sync status data or FALSE, when not syncing:
	* startingBlock: QUANTITY - The block at which the import started (will only be reset, after the sync reached his head)
	* currentBlock: QUANTITY - The current block, same as eth_blockNumber
	* highestBlock: QUANTITY - The estimated highest block

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "net_peerCount",
	"params": [],
	"id": 2
}
```


Response


```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": {
    startingBlock: '0x384',
    currentBlock: '0x386',
    highestBlock: '0x454'
  }
}
```

Or when not syncing
```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": false
}

```

#### Method `eth_getLogs`
* `eth_getLogs(filter)`
    * `filter`: [`Web3Filter`](#type-Web3Filter)
* result: `Arrary`[`Web3Log`](#type-Web3Log) `|`

Returns an array of all logs matching a given filter object.


##### Params

* Object - The filter options:
	* fromBlock: QUANTITY|TAG - (optional, default: "latest") Integer block number, or "latest" for the last mined block or "pending", "earliest" for not yet mined transactions.
	* toBlock: QUANTITY|TAG - (optional, default: "latest") Integer block number, or "latest" for the last mined block or "pending", "earliest" for not yet mined transactions.
	* address: DATA|Array, 20 Bytes - (optional) Contract address or a list of addresses from which logs should originate.
	* topics: Array of DATA, - (optional) Array of 32 Bytes DATA topics. Topics are order-dependent. Each topic can also be an array of DATA with "or" options.
	* blockhash: DATA, 32 Bytes - (optional, future) With the addition of EIP-234, blockHash will be a new filter option which restricts the logs returned to the single block with the 32-byte hash blockHash. Using blockHash is equivalent to fromBlock = toBlock = the block number with hash blockHash. If blockHash is present in in the filter criteria, then neither fromBlock nor toBlock are allowed.

##### Returns

* Object An object with web3 log data.
	

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_getLogs",
	"params": [
		{
    topics: [
      "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    ],
  },
	],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": [{
		"address": "0xf44bb5018bfa4a65165595f5e41f4c7456cd3575",
		"topics": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef", "0x000000000000000000000000f44bb5018bfa4a65165595f5e41f4c7456cd3575", "0x0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e8915"],
		"data": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"blockHash": "0xfbda8b37d5d004653dc18623db2467836b64b24d4880ba4e49ba18a1668f440e",
		"blockNumber": "0x5d54",
		"transactionHash": "0x12f6f4c0cbf388f87ef3d54e8e9a4988746edd9cf773895997cd355f74e2f635",
		"transactionIndex": "0x0",
		"logIndex": "0x0",
		"removed": false
	}, {
		"address": "0xf44bb5018bfa4a65165595f5e41f4c7456cd3575",
		"topics": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef", "0x000000000000000000000000f44bb5018bfa4a65165595f5e41f4c7456cd3575", "0x0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e8915"],
		"data": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"blockHash": "0xfbda8b37d5d004653dc18623db2467836b64b24d4880ba4e49ba18a1668f440e",
		"blockNumber": "0x5d54",
		"transactionHash": "0x5829bfbd0a1752441c5eaca366506757684b99071402448c39c9ae8c1f48b104",
		"transactionIndex": "0x1",
		"logIndex": "0x0",
		"removed": false
	}],
	"id": 2
}
```

#### Method `web3_clientVersion`
* `web3_clientVersion()`  
* result: [`String`](#type-String) 

Returns the current client version.


##### Params

* None

##### Returns

String - The current client version

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "web3_clientVersion",
	"params": [],
	"id": 2
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0.1.0",
	"id": 2
}
```
#### Method `eth_accounts`
* `eth_accounts()`   
* result: `Array<`[`Hex`](#type-Hex) `>`

Returns a list of addresses owned by client.


##### Params

* None

##### Returns

Array of DATA, 20 Bytes - addresses owned by the client.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_accounts",
	"params": [],
	"id": 2
}
```


Response


```
{
  "id":1,
  "jsonrpc": "2.0",
  "result": ["0x407d73d8a49eeb85d32cf465507dd71d507100c1"]
}
```

#### Method `web3_sha3`
* `web3_sha3(data)`
    * `data`: [`Hex`](#type-Hex)
* result: [`Hash`](#type-Hash)

Returns Keccak-256 (not the standardized SHA3-256) of the given data.


##### Params

*   `data` - the data to convert into a SHA3 hash

##### Returns

 DATA - The SHA3 result of the given string.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "web3_sha3",
	"params": ["0x68656c6c6f20776f726c64"],
	"id": 64
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad",
	"id": 2
}

```

#### Method `eth_getStorageAt`
* `eth_getStorageAt(address,position,number)`
    * `address`: [`H160`](#type-H160)
    * `position`: [`U256`](#type-U256)
    * `number`: [`BlockId`](#type-BlockId)
* result: [`Hex`](#type-Hex) 

Returns the value from a storage position at a given address.


##### Params

*   `address` - 20 Bytes - address of the storage.
*   `position` -  integer of the position in the storage.
*   `number` - integer block number, or the string "latest", "earliest" or "pending".
##### Returns

  The value at this storage position.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "web3_sha3",
	"params": [
	 "0x295a70b2de5e3953354a6a8344e616ed314d7251", 
     "0x0",
     "latest"
	],
	"id": 64
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": "0x0000000000000000000000000000000000000000000000000000000000000000",
	"id": 2
}

```

#### Method `eth_coinbase`
* `eth_coinbase()`
* result: [`H160`](#type-H160) 

Returns the client coinbase address.


##### Params

*   None
##### Returns

  DATA, 20 bytes - the current coinbase address.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_coinbase",
	"params": [
	],
	"id": 64
}
```


Response


```
{
  "id":64,
  "jsonrpc": "2.0",
  "result": "0x407d73d8a49eeb85d32cf465507dd71d507100c1"
}


```

#### Method `eth_hashrate`
* `eth_hashrate()`
* result: [`U256`](#type-U256) 

Returns the number of hashes per second that the node is mining with.


##### Params

*   None
##### Returns

  QUANTITY - number of hashes per second.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "eth_hashrate",
	"params": [
	],
	"id": 64
}
```


Response


```
{
  "id":64,
  "jsonrpc": "2.0",
  "result": "0x38a"
}


```



## RPC Types

### Type `BlockView`

The JSON view of a Block including header and body.

#### Fields

`BlockView` is a JSON object with the following fields.

*   `header`: [`HeaderView`](#type-headerview) - The block header.

*   `uncles`: `Array<` [`UncleBlockView`](#type-uncleblockview) `>` - The uncles blocks in the block body.

*   `transactions`: `Array<` [`TransactionView`](#type-transactionview) `>` - The transactions in the block body.

### Type `BlockId`

Consecutive block number starting from 0.

This is a 64-bit unsigned integer type encoded as the 0x-prefixed hex string in JSON. See examples of [Uint64](#type-uint64).


### Type `H256`

The 32-byte fixed-length binary data.

The name comes from the number of bits in the data.

In JSONRPC, it is encoded as a 0x-prefixed hex string.

#### Examples

```
0x696447c51fdb84d0e59850b26bc431425a74daaac070f2b14f5602fbb469912a
```

### Type `HeaderView`

The JSON view of a Header.

This structure is serialized into a JSON object with field `hash` and all the fields in [`Header`](#type-header).

##### Examples


```
{
 		"hash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"parentHash": "0xd619791daa02617ae2825a4ad7f2eb1379a069ac7b96b1628e75e1e654d5163c",
		"sha3Uncles": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"author": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"miner": "0xf4cc1652dcec2e5de9ce6fb1b6f9fa9456e957f1",
		"stateRoot": "0xbd8e37758ba2c1d73f0c1dc3ea255f5b7c037f0e8e1ee9feb012461d58be236d",
		"transactionsRoot": "0x30fe4f21201c335b4501d517872d4d26bec39d350f987ee94e46e27bf7c48aae",
		"receiptsRoot": "0x77f8178b9f5a0e4aa59ee10d3d96cacfd0d6137fd3728cc01b5e5d6d74f6813f",
		"number": "0x1b4",
		"gasUsed": "0xc665442",
		"gasLimit": "0x1c9c380",
		"extraData": "0x",
		"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000",
		"timestamp": "0x62e00014",
		"difficulty": "0x1",
		"totalDifficulty": null,
		"sealFields": [],
		"baseFeePerGas": "0x539"
}
```


#### Fields

`HeaderView` is a JSON object with the following fields.

*   `inner`: [`Header`](#type-header) - All the fields in `Header` are included in `HeaderView` in JSON.

*   `hash`: [`H256`](#type-h256) - The header hash. It is also called the block hash.


### Type `TransactionView`

The JSON view of a Transaction.

This structure is serialized into a JSON object with field `hash` and all the fields in [`Transaction`](#type-transaction).

##### Examples


```
{
	"jsonrpc": "2.0",
	"result": {
		"type": "0x2",
		"blockNumber": "0x1b4",
		"blockHash": "0x9a13208ce76c32638f509064545765c8341db9178b77b4f47b458a66325494fd",
		"hash": "0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65",
		"nonce": "0x1",
		"transactionIndex": "0x1b70",
		"from": "0x92df69a492c93d22c90247434b8d80944daa38fa",
		"to": "0xef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca",
		"value": "0x0",
		"gas": "0x73a9",
		"gasPrice": "0x77359400",
		"maxFeePerGas": "0x539",
		"maxPriorityFeePerGas": "0x77359400",
		"raw": "0x02f8af05018477359400847735940082ea6094ef11d1c2aa48826d4c41e54ab82d1ff5ad8a64ca80b844a9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000c080a0fa9bc76185d06c6e3178c66c40c195f374e80a78179392a84c1db731ce4d2d3da06079373330aa2c6d420267d83d8cd685db20638c7935f02a26fdf99dd010bfa2",
		"input": "0xa9059cbb0000000000000000000000005cf83df52a32165a7f392168ac009b168c9e89150000000000000000000000000000000000000000000000000000000000000000",
		"publicKey": "0x1113b46c2c52050fb9fd29cf88d4a9f8d253e17515dc142f72d8bb2cd18fdb3e45098e11811bcdd1487bafb7b24bf35c174cf6ede558059e23ad60d312345678",
		"accessList": [],
		"chainId": "0x5",
		"v": "0x0",
		"r": "0xfa9bc76185d06c6e3178c66c40c195f374e80a78179392a84c1db731ce4d2d3d",
		"s": "0x6079373330aa2c6d420267d83d8cd685db20638c7935f02a26fdf99dd010bfa2"
	},
	"id": 2
}
```