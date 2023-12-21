# Axon JSON-RPC Protocols
This section lists the Axon JSON-RPC API endpoints. 

Axon JSON-RPC allow you to interact with a local or remote axon node using HTTP, IPC or WebSocket.

## JSONRPC Deprecation Process




## Table of Contents

- [Axon JSON-RPC Protocols](#axon-json-rpc-protocols)
	- [JSONRPC Deprecation Process](#jsonrpc-deprecation-process)
	- [Table of Contents](#table-of-contents)
	- [RPC Methods](#rpc-methods)
		- [Gossip-Methods](#gossip-methods)
			- [Method `eth_sendRawTransaction`](#method-eth_sendrawtransaction)
				- [Params](#params)
				- [Returns](#returns)
				- [Examples](#examples)
			- [Method `eth_blockNumber`](#method-eth_blocknumber)
				- [Params](#params-1)
				- [Returns](#returns-1)
				- [Examples](#examples-1)
			- [Method `eth_submitWork`](#method-eth_submitwork)
				- [Params](#params-2)
				- [Returns](#returns-2)
				- [Examples](#examples-2)
			- [Method `eth_submitHashrate`](#method-eth_submithashrate)
				- [Params](#params-3)
				- [Returns](#returns-3)
				- [Examples](#examples-3)
		- [History-Methods](#history-methods)
			- [Method `eth_getBlockByHash`](#method-eth_getblockbyhash)
				- [Params](#params-4)
				- [Returns](#returns-4)
				- [Examples](#examples-4)
			- [Method `eth_getTransactionByHash`](#method-eth_gettransactionbyhash)
				- [Params](#params-5)
				- [Returns](#returns-5)
				- [Examples](#examples-5)
			- [Method `eth_getBlockTransactionCountByNumber`](#method-eth_getblocktransactioncountbynumber)
				- [Params](#params-6)
				- [Returns](#returns-6)
				- [Examples](#examples-6)
			- [Method `eth_getTransactionReceipt`](#method-eth_gettransactionreceipt)
				- [Params](#params-7)
				- [Returns](#returns-7)
				- [Examples](#examples-7)
			- [Method `eth_feeHistory`](#method-eth_feehistory)
				- [Params](#params-8)
				- [Returns](#returns-8)
				- [Examples](#examples-8)
			- [Method `eth_getBlockTransactionCountByHash`](#method-eth_getblocktransactioncountbyhash)
				- [Params](#params-9)
				- [Returns](#returns-9)
				- [Examples](#examples-9)
			- [Method `eth_getTransactionByBlockHashAndIndex`](#method-eth_gettransactionbyblockhashandindex)
				- [Params](#params-10)
				- [Returns](#returns-10)
				- [Examples](#examples-10)
			- [Method `eth_getTransactionByBlockNumberAndIndex`](#method-eth_gettransactionbyblocknumberandindex)
				- [Params](#params-11)
				- [Returns](#returns-11)
				- [Examples](#examples-11)
			- [Method `eth_getBlockByNumber`](#method-eth_getblockbynumber)
				- [Params](#params-12)
				- [Returns](#returns-12)
				- [Examples](#examples-12)
			- [Method `eth_getProof`](#method-eth_getproof)
				- [Params](#params-13)
				- [Returns](#returns-13)
				- [Examples](#examples-13)
			- [Method `axon_getBlockById`](#method-axon_getblockbyid)
				- [Params](#params-14)
				- [Returns](#returns-14)
				- [Examples](#examples-14)
			- [Method `axon_getProofById`](#method-axon_getproofbyid)
				- [Params](#params-15)
				- [Returns](#returns-15)
				- [Examples](#examples-15)
			- [Method `axon_getMetadataByNumber`](#method-axon_getmetadatabynumber)
				- [Params](#params-16)
				- [Returns](#returns-16)
				- [Examples](#examples-16)
			- [Method `axon_getProposalByNumber`](#method-axon_getproposalbynumber)
				- [Params](#params-17)
				- [Returns](#returns-17)
				- [Examples](#examples-17)
			- [Method `axon_getCkbRelatedInfo`](#method-axon_getckbrelatedinfo)
				- [Params](#params-18)
				- [Returns](#returns-18)
				- [Examples](#examples-18)
		- [State-Methods](#state-methods)
			- [Method `eth_getTransactionCount`](#method-eth_gettransactioncount)
				- [Params](#params-19)
				- [Returns](#returns-19)
				- [Examples](#examples-19)
			- [Method `eth_getBalance`](#method-eth_getbalance)
				- [Params](#params-20)
				- [Returns](#returns-20)
				- [Examples](#examples-20)
			- [Method `eth_chainId`](#method-eth_chainid)
				- [Params](#params-21)
				- [Returns](#returns-21)
				- [Examples](#examples-21)
			- [Method `net_version`](#method-net_version)
				- [Params](#params-22)
				- [Returns](#returns-22)
				- [Examples](#examples-22)
			- [Method `eth_call`](#method-eth_call)
				- [Params](#params-23)
				- [Returns](#returns-23)
				- [Examples](#examples-23)
			- [Method `eth_estimateGas`](#method-eth_estimategas)
				- [Params](#params-24)
				- [Returns](#returns-24)
				- [Examples](#examples-24)
			- [Method `eth_getCode`](#method-eth_getcode)
				- [Params](#params-25)
				- [Returns](#returns-25)
				- [Examples](#examples-25)
			- [Method `eth_gasPrice`](#method-eth_gasprice)
				- [Params](#params-26)
				- [Returns](#returns-26)
				- [Examples](#examples-26)
			- [Method `net_listening`](#method-net_listening)
				- [Params](#params-27)
				- [Returns](#returns-27)
				- [Examples](#examples-27)
			- [Method `eth_mining`](#method-eth_mining)
				- [Params](#params-28)
				- [Returns](#returns-28)
				- [Examples](#examples-28)
			- [Method `net_peerCount`](#method-net_peercount)
				- [Params](#params-29)
				- [Returns](#returns-29)
				- [Examples](#examples-29)
			- [Method `eth_syncing`](#method-eth_syncing)
				- [Params](#params-30)
				- [Returns](#returns-30)
				- [Examples](#examples-30)
			- [Method `eth_getLogs`](#method-eth_getlogs)
				- [Params](#params-31)
				- [Returns](#returns-31)
				- [Examples](#examples-31)
			- [Method `web3_clientVersion`](#method-web3_clientversion)
				- [Params](#params-32)
				- [Returns](#returns-32)
				- [Examples](#examples-32)
			- [Method `eth_accounts`](#method-eth_accounts)
				- [Params](#params-33)
				- [Returns](#returns-33)
				- [Examples](#examples-33)
			- [Method `web3_sha3`](#method-web3_sha3)
				- [Params](#params-34)
				- [Returns](#returns-34)
				- [Examples](#examples-34)
			- [Method `eth_getStorageAt`](#method-eth_getstorageat)
				- [Params](#params-35)
				- [Returns](#returns-35)
				- [Examples](#examples-35)
			- [Method `eth_coinbase`](#method-eth_coinbase)
				- [Params](#params-36)
				- [Returns](#returns-36)
				- [Examples](#examples-36)
			- [Method `eth_hashrate`](#method-eth_hashrate)
				- [Params](#params-37)
				- [Returns](#returns-37)
				- [Examples](#examples-37)
			- [Method `axon_getCurrentMetadata`](#method-axon_getcurrentmetadata)
				- [Params](#params-38)
				- [Returns](#returns-38)
				- [Examples](#examples-38)
			- [Method `axon_getHardforkInfo`](#method-axon_gethardforkinfo)
				- [Params](#params-39)
				- [Returns](#returns-39)
				- [Examples](#examples-39)
	- [RPC Types](#rpc-types)
		- [Type `Web3Filter`](#type-web3filter)
			- [Fields](#fields)
		- [Type `Web3Log`](#type-web3log)
			- [Fields](#fields-1)
		- [Type `Web3SyncStatus`](#type-web3syncstatus)
			- [Fields](#fields-2)
		- [Type `Web3CallRequest`](#type-web3callrequest)
			- [Fields](#fields-3)
		- [Type `AccessList`](#type-accesslist)
			- [Fields](#fields-4)
		- [Type `AccessListItem`](#type-accesslistitem)
			- [Fields](#fields-5)
		- [Type `BlockView`](#type-blockview)
			- [Fields](#fields-6)
		- [Type `Web3FeeHistory`](#type-web3feehistory)
			- [Fields](#fields-7)
		- [Type `Web3Receipt`](#type-web3receipt)
			- [Fields](#fields-8)
		- [Type `Web3ReceiptLog`](#type-web3receiptlog)
			- [Fields](#fields-9)
		- [Type `BlockId`](#type-blockid)
		- [Type `H256`](#type-h256)
			- [Examples](#examples-40)
		- [Type `H160`](#type-h160)
			- [Examples](#examples-41)
		- [Type `Hex`](#type-hex)
			- [Examples](#examples-42)
		- [Type `Hash`](#type-hash)
			- [Examples](#examples-43)
		- [Type `String`](#type-string)
			- [Examples](#examples-44)
		- [Type `bool`](#type-bool)
			- [Examples](#examples-45)
		- [Type `f64`](#type-f64)
		- [Type `Bloom`](#type-bloom)
		- [Type `U64`](#type-u64)
		- [Type `U256`](#type-u256)
		- [Type `TransactionView`](#type-transactionview)
			- [Fields](#fields-10)
				- [Examples](#examples-46)


## RPC Methods

### Gossip-Methods

These methods track the head of the chain. This is how transactions make their way around the network, find their way into blocks, and how clients find out about new blocks.


#### Method `eth_sendRawTransaction`
* `eth_sendRawTransaction(data)`
    * `data`: [`Hex`](#type-Hex) 
* result: [`H256`](#type-H256) 

Submits a pre-signed transaction for broadcast to the Axon network.

##### Params

*   `data` - The signed transaction data.


##### Returns

TRANSACTION HASH - 32 Bytes - the transaction hash.

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
* result: [`U256`](#type-U256) 

Returns the current "latest" block number.

##### Params

* None


##### Returns

BLOCK NUMBER - a hex code of an integer representing the current block number the client is on.

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
* result: [`bool`](#type-bool) 

Used for submitting a proof-of-work solution. Since the axon does not use it, so returns true forever.

##### Params

*   `nc` - 8 Bytes - The nonce found (64 bits)
*   `hash` -  32 Bytes - The header's pow-hash (256 bits)
*   `summary` - 32 Bytes - The mix digest (256 bits)
##### Returns

  Boolean - returns true if the provided solution is valid, otherwise false.Since the axon does not use it, so return true forever.

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
* result: [`bool`](#type-bool) 

Used for submitting mining hashrate. This method always returns true.

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



### History-Methods
Fetches historical records of every block back to genesis. This is like one large append-only file, and includes all block headers, block bodies, uncle blocks, and transaction receipts.

#### Method `eth_getBlockByHash`
* `eth_getBlockByHash(hash,show_rich_tx)`
    * `hash`: [`H256`](#type-H256) 
    * `show_rich_tx`: [`bool`](#type-bool)
* result: [`BlockView`](#type-BlockView)

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
* `eth_getTransactionByHash(txHash)`
    * `txHash`: [`H256`](#type-H256)
* result: [`TransactionView`](#type-TransactionView)

Returns the information about a transaction requested by transaction hash.


##### Params

*   `txHash` - Hash of a transaction.


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
* result: [`U256`](#type-U256)

Returns the number of transactions in a block matching the given block number.




##### Params

*   `number` - A block number.

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
* result: [`Web3Receipt`](#type-Web3Receipt)

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
	* `reward_percentiles`: `Array<`[`f64`](#type-f64)`>`
* result: [`Web3FeeHistory`](#type-Web3FeeHistory)


Returns base fee per gas and transaction effective priority fee per gas history for the requested block range if available. The range between headBlock-4 and headBlock is guaranteed to be available while retrieving data from the pending block and older history are optional to support. For pre-EIP-1559 blocks, the gas prices are returned as rewards and zeroes are returned for the base fee per gas.
blockCount and newestBlock are both required parameters


##### Params

*   `block_count` - 256-bit unsigned integer.
*   `newest_block` - A block number.
*   `reward_percentiles` -  (optional) A monotonically increasing list of percentile values to sample from each block's effective priority fees per gas in ascending order, weighted by gas used.


##### Returns
* oldestBlock - Lowest number block of the returned range expressed as a hexidecimal number.
* baseFeePerGas - An array of block base fees per gas. This includes the next block after the newest of the returned range, because this value can be derived from the newest block. Zeroes are returned for pre-EIP-1559 blocks.
* gasUsedRatio - An array of block gas used ratios. These are calculated as the ratio of gasUsed and gasLimit.
* reward - An array of effective priority fee per gas data points from a single block. All zeroes are returned if the block is empty.

##### Examples

Request


```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_feeHistory",
  "params": [
    "0x1",
	"latest"
	[20, 30],
  ]
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"oldestBlock": "0x0",
		"baseFeePerGas": ["0x539","0x539"],
		"gasUsedRatio": [0.0],
		"reward": [["0x0"]]
	},
	"id": 2
}

```

#### Method `eth_getBlockTransactionCountByHash`
* `eth_getBlockTransactionCountByHash(hash)`
    * `hash`: [`Hash`](#type-Hash)
* result: [`U256`](#type-U256) 

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

*   `number` -  A block number.
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



#### Method `eth_getBlockByNumber`
* `eth_getBlockByNumber(number,show_rich_tx)`
    * `number`: [`BlockId`](#type-BlockId) `|` `"earliest"``|` `"latest"``|` `"pending"`
    * `show_rich_tx`: [`bool`](#type-bool)
* result: [`BlockView`](#type-Blockview) 

Returns information about a block by block number.


##### Params

*   `number` -  A block number.
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

#### Method `eth_getProof`

##### Params

*   `DATA`, 20 Bytes - address of the account.
*   `ARRAY`, 32 Bytes - array of storage-keys which should be proofed and included. See eth_getStorageAt
*   `QUANTITY|TAG` - integer block number, or the string "latest" or "earliest", see the default block parameter

##### Returns

Object - A account object:

*   `balance`: `QUANTITY` - the balance of the account.
*   `codeHash`: `DATA`, 32 Bytes - hash of the code of the account. For a simple Account without code it will return "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
*   `nonce`: `QUANTITY`, - nonce of the account.
*   `storageHash`: `DATA`, 32 Bytes - SHA3 of the StorageRoot. All storage will deliver a MerkleProof starting with this rootHash.
*   `accountProof`: `ARRAY` - Array of rlp-serialized MerkleTree-Nodes, starting with the stateRoot-Node, following the path of the SHA3 (address) as key.

*   `storageProof`: `ARRAY` - Array of storage-entries as requested. Each entry is a object with these properties:
        `key`: `QUANTITY` - the requested storage key
        `value`: `QUANTITY` - the storage value
        `proof`: `ARRAY` - Array of rlp-serialized MerkleTree-Nodes, starting with the storageHash-Node, following the path of the SHA3 (key) as path.

##### Examples

Request
```
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "eth_getProof",
  "params": [
    "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
    [  "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421" ],
    "latest"
  ]
}
```

Response
```

{
    "jsonrpc":"2.0",
    "result":{
        "balance":"0x0",
        "codeHash":"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
        "nonce":"0x1",
        "storageHash":"0x6fcd3e8e97da273711ccefb79abdd246c5663c7d61057f82bae745ceac5dcc75",
        "accountProof":[
            "0xf9011180a063017aec481298bbc5df01d044f2a85b25f35f565864f285f88355b48332e07da0b01d7c543eaa8e9ae61ef41d6a73e4bc00010807523108be4146c01fb2812163a011deb9627b74106f9b5a605e9a5d7982e217ab4fde0acd3cf40def9010470373808080a06cd5cbddd2b22b17026df25fe3153d555656298115795903ecaa813684bc21e7a021c9bcb004c74f63f386e6cf04a0c59d425c5f8c9e28e9bf172402b431aa249ca005550a2b2f338a7daa477729d59ddecc3852e2821d920b0e4414f801abee27b8a01d65e6a5eb57de1c0b7990e75ed2f39ecb56ed7b18182d00ada8f2088893c32c80808080a0c8487d369704a81bd26ff351c89f333ae51a6adf7676739ba0995f6f97c6b31080",
            "0xf85d943ab0cf264df99d83525e9e11c7e4db01558ae1b1b846f8440180a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        ],
        "storageProof":[
			{
				"key" : "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"proof" : [],
				"value" : "0x0"
        	}
        ]
    },
    "id":1
}

```

#### Method `axon_getBlockById`

##### Params

*   `QUANTITY|TAG` - integer block number, or the string "latest", but not "earliest", see the default block parameter

##### Returns

The RPC returns the axon block details, not web3 compatible.

##### Examples

Request
```
{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "axon_getBlockById",
    "params": ["latest"]
}
```

Response
```
{
    "jsonrpc":"2.0",
    "result":{
        "header":{
            "version":"V0",
            "prev_hash":"0xc7cc43119b4e12869ff9e43a16c7f34520232752c7c50a09256895389f5fc2b2",
            "proposer":"0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
            "state_root":"0xd91b19ba782e3a278db25878e02f4e52d4a3e49f2beae20589a0d68de984cbfb",
            "transactions_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "signed_txs_hash":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "receipts_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "log_bloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000001000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "timestamp":"0x6583df6b",
            "number":"0x1ce",
            "gas_used":"0x0",
            "gas_limit":"0x1c9c380",
            "extra_data":[
            ],
            "base_fee_per_gas":"0x539",
            "proof":{
                "number":"0x1cd",
                "round":"0x0",
                "proposal_hash":"0xce3b8b5f649e77b50eccbf2c117c481e3248aa6312863a9eeedba077bcb2b83c",
                "signature":"0xab61d43e7cae2e0a5edaa5569a4f195bc2e759e6689f941a38cb47f927c5ac09116be9699107084c007b7caab6bf3b2614aa0f96440ef1d02766a41018294a7f92b5954d33f6539c4a186285de18ca998c6fb829cf1e3721b8ddd1665790025d",
                "bitmap":"0x80"
            },
            "call_system_script_count":"0x0",
            "chain_id":"0x41786f6e"
        },
        "tx_hashes":[
        ]
    },
    "id":2
}
```

#### Method `axon_getProofById`

##### Params

*   `QUANTITY|TAG` - integer block number, or the string "latest", but not "earliest", see the default block parameter

##### Returns

The RPC returns the axon block proof.

##### Examples

Request
```
{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "axon_getProofById",
    "params": ["latest"]
}
```

Response
```
{
    "jsonrpc":"2.0",
    "result":{
        "number":"0x1d1",
        "round":"0x0",
        "proposal_hash":"0x42c8210c9e8df7a8d13bfcaf00d4e3f87abbb3747ac38a1cd82f9571471d0536",
        "signature":"0xb624b749aafd1af30736097fc3d87caa55b4f2d18382c2f34522b01bfbb7d5ef75986ddf0f4db01d0b804459396c07480908a3a3c0f46eb2e16ab99d4cc81473bb60b25bac96924d9342b66bceca927fdf407209a07af7426a3cb179e8c11f91",
        "bitmap":"0x80"
    },
    "id":2
}
```

#### Method `axon_getMetadataByNumber`

##### Params

* `BlockNumber`: U256

##### Returns

Metadata struct

##### Examples

Request
```
{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "axon_getMetadataByNumber",
    "params": ["0x10"]
}
```

Response
```
{
    "jsonrpc":"2.0",
    "result":{
        "version":{
            "start":"0x1",
            "end":"0x5f5e100"
        },
        "epoch":"0x0",
        "verifier_list":[
            {
                "bls_pub_key":"0xa26e3fe1cf51bd4822072c61bdc315ac32e3d3c2e2484bb92942666399e863b4bf56cf2926383cc706ffc15dfebc85c6",
                "pub_key":"0x031ddc35212b7fc7ff6685b17d91f77c972535aee5c7ae5684d3e72b986f08834b",
                "address":"0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
                "propose_weight":"0x1",
                "vote_weight":"0x1"
            }
        ],
        "propose_counter":[
            {
                "address":"0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
                "count":"0x10"
            }
        ],
        "consensus_config":{
            "gas_limit":"0x3e7fffffc18",
            "interval":"0xbb8",
            "propose_ratio":"0xf",
            "prevote_ratio":"0xa",
            "precommit_ratio":"0xa",
            "brake_ratio":"0xa",
            "tx_num_limit":"0x4e20",
            "max_tx_size":"0x186a0000",
            "max_contract_limit":"0x6000"
        }
    },
    "id":2
}
```


#### Method `axon_getProposalByNumber`

##### Params

* `BlockNumber`: U256

##### Returns

Proposal struct

##### Examples

Request
```
{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "axon_getProposalByNumber",
    "params": ["0x10"]
}
```

Response
```
{
    "jsonrpc":"2.0",
    "result":{
        "version":"V0",
        "prev_hash":"0x7c3ed8b18b3214b12b34b3374ad199cdf4854713599e3bf5ec2c75bcf02687bd",
        "proposer":"0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
        "prev_state_root":"0xd2b96eda4834846a228d318f69287b0b5f872ee09ce064a2b6fc8801a6397bc7",
        "transactions_root":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        "signed_txs_hash":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "timestamp":"0x655d96e7",
        "number":"0x10",
        "gas_limit":"0x1c9c380",
        "extra_data":[
        ],
        "base_fee_per_gas":"0x539",
        "proof":{
            "number":"0xf",
            "round":"0x0",
            "proposal_hash":"0xe2df5afbaf2e61b6ef66be946872d549771d46744be3a8521f4945ab49779bcd",
            "signature":"0x9603202565eb0c4fd723e9aa84b4b289f6e546b2a251a1b88ea256f8865381449fc6de3a1f226d1386cb8601b5bf526b117a8858dc989690b1b93429b3332b83ec94d4f0b7f367272cd8e266b887f8fc856e29b1b1276b2691deb3f23aa13382",
            "bitmap":"0x80"
        },
        "chain_id":"0x41786f6e",
        "call_system_script_count":"0x0",
        "tx_hashes":[
        ]
    },
    "id":2
}
```

#### Method `axon_getCkbRelatedInfo`

##### Params

* None

##### Returns

CkbRelatedInfo struct

##### Examples

Request
```
{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "axon_getCkbRelatedInfo",
    "params": []
}
```

Response
```
{
    "jsonrpc":"2.0",
    "result":{
		"metadata_type_id": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"checkpoint_type_id": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"xudt_args": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"stake_smt_type_id": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"delegate_smt_type_id": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"reward_smt_type_id": "0x0000000000000000000000000000000000000000000000000000000000000000"
    },
    "id":2
}
```

### State-Methods
Methods that report the current state of all the data stored. The "state" is like one big shared piece of RAM, and includes account balances, contract data, and gas estimations.

#### Method `eth_getTransactionCount`
* `eth_getTransactionCount(address,number)`
    * `address`: [`H160`](#type-H160)
    * `number`: [`BlockId`](#type-BlockId)
* result: [`U256`](#type-U256)

Returns the number of transactions sent from an address.


##### Params

*   `address` - 20 Bytes - address.
*   `number` - Integer block number

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
    * `number`: [`BlockId`](#type-BlockId)
* result: [`U256`](#type-U256)

Returns the balance of the account of given address.


##### Params

*   `address` - 20 Bytes - address to check for balance.
*   `block_number` - A block number.

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
* result: [`U256`](#type-U256)

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
* result: [`U256`](#type-U256)

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
* result: [`Hex`](#type-Hex)

Executes a new message call immediately without creating a transaction on the block chain.


##### Params

* `req` - The transaction call object

	* from: DATA, 20 Bytes - (optional) The address the transaction is sent from.
	* to: DATA, 20 Bytes - The address the transaction is directed to.
	* gas: QUANTITY - (optional) Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.
	* gasPrice: QUANTITY - (optional) Integer of the gasPrice used for each paid gas
	* value: QUANTITY - (optional) Integer of the value sent with this transaction
	* data: DATA - (optional) Hash of the method signature and encoded parameters.
* `number` - A block number.

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
* result: [`U256`](#type-U256) 

Generates and returns an estimate of how much gas is necessary to allow the transaction to complete. The transaction will not be added to the blockchain. Note that the estimate may be significantly more than the amount of gas actually used by the transaction, for a variety of reasons including EVM mechanics and node performance.


##### Params

* `req` - The transaction call object. Expect all properties are optional. If no gas limit is specified geth uses the block gas limit from the pending block as an upper bound. As a result the returned estimate might not be enough to executed the call/transaction when the amount of gas is higher than the pending block gas limit.

	* from: DATA, 20 Bytes - (optional) The address the transaction is sent from.
	* to: DATA, 20 Bytes - The address the transaction is directed to.
	* gas: QUANTITY - (optional) Integer of the gas provided for the transaction execution. eth_call consumes zero gas, but this parameter may be needed by some executions.
	* gasPrice: QUANTITY - (optional) Integer of the gasPrice used for each paid gas
	* value: QUANTITY - (optional) Integer of the value sent with this transaction
	* data: DATA - (optional) Hash of the method signature and encoded parameters.
* `number` - A block number.

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
* result: [`Hex`](#type-Hex)

Returns code at a given address.


##### Params

* `address` - DATA, 20 Bytes - address.
* `block_number` -A block number.

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
    
* result: [`U256`](#type-U256)

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
    
* result: [`bool`](#type-bool)

Returns true if client is actively listening for network connections. Always return true in Axon network.


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
    
* result: [`bool`](#type-bool)

Returns true if client is actively mining new blocks. 

*Note:* Always return false in Axon network.


##### Params

* None

##### Returns

 Boolean - returns true of the client is mining, otherwise false. Always return false in Axon network.

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
    
* result: [`U256`](#type-U256)

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
* result: `Arrary`[`Web3Log`](#type-Web3Log)

Returns an array of all logs matching a given filter object.


##### Params

* Object - The filter options:
	* fromBlock: QUANTITY|TAG - (optional, default: "latest") Integer block number, or "latest" for the last mined block or "pending", "earliest" for not yet mined transactions.
	* toBlock: QUANTITY|TAG - (optional, default: "latest") Integer block number, or "latest" for the last mined block or "pending", "earliest" for not yet mined transactions.
	* address: DATA|Array, 20 Bytes - (optional) Contract address or a list of addresses from which logs should originate.
	* topics: Array of DATA, - (optional) Array of 32 Bytes DATA topics. Topics are order-dependent. Each topic can also be an array of DATA with "or" options.
	* blockhash: DATA, 32 Bytes - (optional, future) With the addition of EIP-234, blockHash will be a new filter option which restricts the logs returned to the single block with the 32-byte hash blockHash. Using blockHash is equivalent to fromBlock = toBlock = the block number with hash blockHash. If blockHash is present in in the filter criteria, then neither fromBlock nor toBlock are allowed.

##### Returns

* Object An object with web3 log data. See [`Web3Log`](#type-Web3Log).
	

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
* result: `Array<`[`Hex`](#type-Hex)`>`

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
*   `number` - A block number.
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

*Note:* Always return 1 in Axon network.


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

#### Method `axon_getCurrentMetadata`
* `axon_getCurrentMetadata()`
* result: [`Metadata`](#Metadata) 

Returns axon current metadata info.

##### Params

*   None

##### Returns

Returns axon current metadata info.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "axon_getCurrentMetadata",
	"params": [
	],
	"id": 64
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"version": {
			"start": "0x1",
			"end": "0x5f5e100"
		},
		"epoch": "0x0",
		"verifier_list": [{
			"bls_pub_key": "0xa26e3fe1cf51bd4822072c61bdc315ac32e3d3c2e2484bb92942666399e863b4bf56cf2926383cc706ffc15dfebc85c6",
			"pub_key": "0x031ddc35212b7fc7ff6685b17d91f77c972535aee5c7ae5684d3e72b986f08834b",
			"address": "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
			"propose_weight": "0x1",
			"vote_weight": "0x1"
		}],
		"propose_counter": [{
			"address": "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
			"count": "0xb4"
		}],
		"consensus_config": {
			"gas_limit": "0x3e7fffffc18",
			"interval": "0xbb8",
			"propose_ratio": "0xf",
			"prevote_ratio": "0xa",
			"precommit_ratio": "0xa",
			"brake_ratio": "0xa",
			"tx_num_limit": "0x4e20",
			"max_tx_size": "0x186a0000",
			"max_contract_limit": "0x8000"
		}
	},
	"id": 73
}
```

#### Method `axon_getHardforkInfo`
* `axon_getHardforkInfo()`
* result: [`HardforkInfos`](#HardforkInfos) 

Returns axon current hardfork infos.

##### Params

*   None

##### Returns

Returns axon current hardfork infos.

##### Examples

Request


```
{
	"jsonrpc": "2.0",
	"method": "axon_getHardforkInfo",
	"params": [
	],
	"id": 64
}
```


Response


```
{
	"jsonrpc": "2.0",
	"result": {
		"Andromeda":"enabled"
	},
	"id": 73
}
```

## RPC Types

### Type `Web3Filter`

The Web3Filter objects.

#### Fields

`Web3Filter` is a JSON object with the following fields.

*   `from_block`: [`BlockId`](#type-BlockId) - [optional, default is "latest"] 

	hexadecimal block number, or the string "latest", "earliest" or "pending"

*   `to_block`:  [`BlockId`](#type-BlockId) - [optional, default is "latest"] 

	hexadecimal block number, or the string "latest", "earliest" or "pending"

*   `block_hash`: [`H256`](#type-H256) - [optional] With the addition of EIP-234, blockHash 		restricts the logs returned to the single block with the 32-byte hash blockHash. 

	UsingblockHash is equivalent to.    

*   `address`: [`H256`](#type-H256)  - [optional] - a string representing the address (20 bytes) to check for balance

	null when its pending. 

	null when its pending log.

*   `topics`: `Array<`[`Hash`](#type-Hash)`>` - [optional] - Array of 32 Bytes DATA topics. 

	Topics are order-dependent.

### Type `Web3Log`

The Web3Log log objects.

#### Fields

`Web3Log` is a JSON object with the following fields.

*   `address`: [`H160`](#type-H160) - Hex encoded 20 Bytes - address from which this log originated.

*   `topics`: `Array<`[`H256`](#type-H256)`>` - Array of 0 to 4 32 Bytes of indexed log 			arguments. 

	In solidity: The first topic is the hash of the signature of the event (e.g. Deposit(address,bytes32,uint256)), except you declared the event with the anonymous specifier.

*   `data`:  [`Hex`](#type-Hex) - contains one or more 32 Bytes non-indexed arguments of the log.

*   `block_hash`: [`H256`](#type-H256)  - [optional] With the addition of EIP-234, blockHash 		restricts the logs returned to the single block with the 32-byte hash blockHash. 

	UsingblockHash is equivalent to.    

*   `block_number`: [`U256`](#type-U256)  - the block number where this log was in. 

	null when its pending. 

	null when its pending log.

*   `transaction_hash`: [`H256`](#type-H256) - Hex encoded 32 Bytes, hash of the transactions this log 		was created from. null when its pending log..

*   `transaction_index`: [`U256`](#type-U256) - hexadecimal of the transactions index 				position log was created from. 

	null when its pending log.

*   `log_index`: [`U256`](#type-U256)  - hexadecimal of the log index position in the block.

 	null when its pending log.

*   `removed`: [`bool`](#type-bool)  - true when the log was removed, due to a chain				reorganization. false if it's a valid log.


### Type `Web3SyncStatus`

The Web3SyncStatus objects.

#### Fields

`Web3SyncStatus` is a JSON object with the following fields.

*   `starting_block`: [`U256`](#type-U256) - QUANTITY - The block at which the import started (will only be reset, after the sync reached his head)

*   `current_block`:  [`U256`](#type-U256) - QUANTITY - The current block, same as eth_blockNumber.

*   `highest_block`:  [`U256`](#type-U256) -QUANTITY - The estimated highest block.


### Type `Web3CallRequest`

The Web3CallRequest objects.

#### Fields

`Web3CallRequest` is a JSON object with the following fields.

*   `transaction_type`: [`U64`](#type-U64) - 64-bit integer that represents the type of the transaction. Axon has evolved to support 3 types of transactions: Legacy is 0x0, Eip2930  is 0x1 and Eip1559  is 0x2.

*   `from`:[`H160`](#type-H256) - Hex encoded 20 Bytes - The address the transaction is sent from.

*   `to`:  [`H160`](#type-H160) - Hex encoded 20 Bytes - The address the transaction is directed to.

*   `gas_price`: [`U256`](#type-U256)  - [optional] hexadecimal value of the gasPrice used for each paid gas.

*   `max_fee_per_gas`: [`U256`](#type-U256)  - [optional] Maximum total fee (base fee + priority fee), in Wei, the sender is willing to pay per gas.

*   `gas`: [`U256`](#type-U256) -  [optional] hexadecimal value of the gas provided for the 		transaction execution. 

	eth_call consumes zero gas, but this parameter may be needed by some executions.

*   `value`: [`U256`](#type-U256) - [optional] hexadecimal value of the value sent with this 		transaction.

*   `data`: [`Hex`](#type-Hex)  - [optional] Hash of the method signature and encoded parameters.

*   `nonce`: [`U256`](#type-U256)  - Nonce is a sequence number, issued by the originating externally owned account, used to prevent message replay.

*   `access_list`: `Array<` [`AccessList`](#type-AccessList)`>`  - The accessList specifies a list of addresses and storage keys; these addresses and storage keys are added into the accessed_addresses and accessed_storage_keys global sets.

*   `chain_id`: [`U64`](#type-U64)  - QUANTITY - The id of the chain.

*   `max_priority_fee_per_gas`: [`U256`](#type-U256)  -  QUANTITY - (optional) determined by the user, and is paid directly to miners.

### Type `AccessList`

The AccessList objects.

#### Fields

`AccessList` is a JSON object with the following fields.

*   `AccessList`: `Arrar<`[`AccessListItem`]`>`(#type-AccessListItem) - A list of addresses and storage keys.


### Type `AccessListItem`

The AccessListItem objects.

#### Fields

`AccessListItem` is a JSON object with the following fields.

*   `address`: [`H160`](#type-H160) - DATA, Hex encoded 20 Bytes - The addresses is added into the accessed_addresses global sets.

*   `storage_keys`: `Arrar<`[`H256`]`>`(#type-H256) - DATA, Hex encoded 32 Bytes - The storage keys is added into the accessed_storage_keys global sets.


### Type `BlockView`

The BlockView objects.

#### Fields

`BlockView` is a JSON object with the following fields.

*   `hash`: [`H256`](#type-H256) -  DATA, Hex encoded 32 Bytes - hash of the block. null when its pending block.

*   `parent_hash`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - hash of the parent block.

*   `sha3_uncles`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - SHA3 of the uncles data in the block.

*   `author`: [`H169`](#type-H160)  - Hex encoded 20 Bytes - the creator of the block.

*   `miner`: [`H160`](#type-H160)  - DATA, Hex encoded 20 Bytes - the address of the beneficiary to whom the mining rewards were given.

*   `state_root`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - the root of the final state trie of the block.

*   `transactions_root`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - the root of the transaction trie of the block.

*   `receipts_root`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - the root of the receipts trie of the block.

*   `number`: [`U256`](#type-U256)  - QUANTITY - the block number. null when its pending block.

*   `gas_used`: [`U256`](#type-U256)  - QUANTITY - the total used gas by all transactions in this block.

*   `gas_limit`: [`U256`](#type-U256)  - QUANTITY - the maximum gas allowed in this block.

*   `extra_data`: [`Hex`](#type-Hex)  - DATA - the "extra data" field of this block.

*   `logs_bloom`: [`Boolom`](#type-Boolom)  - DATA, 256 Bytes - the bloom filter for the logs of the block. null when its pending block.

*   `timestamp`: [`U256`](#type-U256)  - QUANTITY - the unix timestamp for when the block was collated.

*   `difficulty`: [`U256`](#type-U256)  - QUANTITY - integer of the difficulty for this block.

*   `total_difficulty`: [`U256`](#type-U256)  - QUANTITY - integer of the total difficulty of the chain until this block.

*   `base_fee_per_gas`: [`U256`](#type-U256)  - he minimum fee per gas required for a transaction to be included in the block.

*   `uncles`: `Array<`[`U256`](#type-U256)`>`  - Array - Array of uncle hashes.

*   `transactions`:`Array<`[`RichTransactionOrHash`](#type-RichTransactionOrHash)`>` - The transactions in the block body.

*   `size`: [`U256`](#type-U256)  - QUANTITY - integer the size of this block in bytes.

*   `mix_hash`: [`H256`](#type-H256)  - Hex encoded 32 Bytes -  a unique identifier for that block.

*   `nonce`: [`U256`](#type-U256)  - DATA, 8 Bytes - hash of the generated proof-of-work. null when its pending block.


### Type `Web3FeeHistory`

The Web3FeeHistory objects.

#### Fields

`Web3FeeHistory` is a JSON object with the following fields.

*   `oldest_block`: [`U256`](#type-U256) - Lowest number block of the returned range.

*   `reward`:  `Arrary<`[`U256`](#type-U256)`>` - (Optional) An array of effective priority fee per gas data points from a single block. All zeroes are returned if the block is empty.

*   `base_fee_per_gas`:  `Arrary<`[`U256`](#type-U256)`>` - An array of block base fees per gas. 	This includes the next block after the newest of the returned range, because this value can be derived from the newest block. 
	Zeroes are returned for pre-EIP-1559 blocks.

*   `gas_used_ratio`:  `Arrary<`[`U256`](#type-U256)`>` - An array of block gas used ratios. 		These are calculated as the ratio of gasUsed and gasLimit.

### Type `Web3Receipt`

The Web3Receipt objects.

#### Fields

`Web3Receipt` is a JSON object with the following fields.

*   `block_number`: [`U256`](#type-U256) - QUANTITY - block number where this transaction was in.

*   `block_hash`: [`H256`](#type-H256) - DATA, Hex encoded 32 Bytes - hash of the block where this transaction was in.

*   `contract_address`: [`H160`](#type-H160) - DATA, Hex encoded 20 Bytes - The contract address created, if the transaction was a contract creation, otherwise null.

*   `cumulative_gas_used`: [`U256`](#type-U256) - QUANTITY - The total amount of gas used when this transaction was executed in the block.

*   `effective_gas_price`: [`U256`](#type-U256) - QUANTITY - the price per gas at the time of your transaction, so the total gas cost of your transaction is effectiveGasPrice * gasUsed

*   `from`: [`H160`](#type-H160) - DATA, Hex encoded 20 Bytes - address of the sender.

*   `gas_used`: [`U256`](#type-U256) - QUANTITY - The amount of gas used by this specific transaction alone.

*   `logs`:  `Arrary<`[`Web3ReceiptLog`](#type-Web3ReceiptLog)`>` - Array - Array of log objects, which this transaction generated.

*   `logs_bloom`: [`Bloom`](#type-Bloom) - DATA, 256 Bytes - Bloom filter for light clients to quickly retrieve related logs.

*   `state_root`: [`Hash`](#type-Hash) - DATA 32 bytes of post-transaction stateroot (pre Byzantium)

*   `status`: [`U256`](#type-U256) - QUANTITY either 1 (success) or 0 (failure)

*   `to`: [`H160`](#type-H160)- DATA, Hex encoded 20 Bytes - address of the receiver. null when its a contract creation transaction.

*   `transaction_hash`: [`Hash`](#type-Hash) - DATA, Hex encoded 32 Bytes - hash of the transaction.

*   `transaction_index`: [`U256`](#type-U256) - QUANTITY - integer of the transactions index position in the block.

*   `transaction_type`: [`U64`](#type-U64) - 64-bit integer that represents the type of the transaction. Axon has evolved to support 3 types of transactions: Legacy is 0x0, Eip2930  is 0x1 and Eip1559  is 0x2.


### Type `Web3ReceiptLog`

The Web3ReceiptLog objects.

#### Fields

`Web3ReceiptLog` is a JSON object with the following fields.

*   `address`: [`H160`](#type-H160) - DATA, Hex encoded 20 Bytes - address from which this log originated.

*   `topics`: `Arrary<`[`H256`](#type-H256)`>` - rray of DATA - Array of 0 to 4 Hex encode 32 Bytes DATA of indexed log arguments.
	* In solidity: The first topic is the hash of the signature of the event (e.g. Deposit(address,bytes32,uint256)), except you declare the event with the anonymous specifier.

*   `data`: [`Hex`](#type-Hex)`>` - DATA - contains one or more 32 Bytes non-indexed arguments of the log.

*   `block_number`: [`U256`](#type-U256) - QUANTITY - the block number where this log was in. null when its pending. null when its pending log.

*   `block_hash`: [`Hash`](#type-Hash) - DATA, Hex encode 32 Bytes - hash of the block where this log was in. null when its pending. null when its pending log.

*   `transaction_hash`: [`Hash`](#type-Hash)`>` - DATA, Hex encoded 32 Bytes - hash of the transactions this log was created from. null when its pending log.

*   `transaction_index`: [`U256`](#type-U256) - QUANTITY - integer of the transactions index position log was created from. null when its pending log.

*   `log_index`:  [`U256`](#type-U256)`>` - QUANTITY - integer of the log index position in the block. null when its pending log.

*   `removed`: [`bool`](#type-bool) - TAG - true when the log was removed, due to a chain reorganization. false if its a valid log.


### Type `BlockId`

Default block parameters can be one of the following:

* Number|BN|BigNumber: A block number starting from 0.This is a 64-bit unsigned integer type encoded as the 0x-prefixed hex string in JSON.

* "earliest" - String: The genesis block

* "latest" - String: The latest block (current head of the blockchain)

* "pending" - String: The currently mined block (including pending transactions)


### Type `H256`

The Hex encoded 32-bytes fixed-length binary data.

The name comes from the number of bits in the data.


#### Examples

```
0x696447c51fdb84d0e59850b26bc431425a74daaac070f2b14f5602fbb469912a
```

### Type `H160`

Fixed-size uninterpreted hash type with hex encoded 20 bytes (160 bits) size.

#### Examples

```
0x92df69a492c93d22c90247434b8d80944daa38fa
```

### Type `Hex`

0x-prefixed string in JSON.

#### Examples

```
0x0 // Decimal Value is 0
0x10 // Decimal Value is 16
10 // Invalid, 0x is required
```

### Type `Hash`

The 32-byte fixed-length binary data.In JSONRPC, it is Hex encoded as a 0x-prefixed hex string.

#### Examples

```
0x41e946c6f4dd97ad2828c056af973087b53044bf567caf0ea870ab45460afd65
```

### Type `String`

A UTF-8–encoded, growable string.

The String type is the most common string type that has ownership over the contents of the string. It has a close relationship with its borrowed counterpart, the primitive str.

#### Examples

```
"0.1.0"
```

### Type `bool`

The boolean type.

The bool represents a value, which could only be either true or false. If you cast a bool into an integer, true will be 1 and false will be 0.

#### Examples

```
true
```
or
```
1
```

### Type `f64`

A 64-bit floating point type (specifically, the "binary64" type defined in IEEE 754-2008).

This type is very similar to f32, but has increased precision by using twice as many bits. Please see the documentation for f32 or Wikipedia on double precision values for more information.

### Type `Bloom`

Bloom hash type with 256 bytes (2048 bits) size.

### Type `U64`

Unsigned 64-bit integer.

### Type `U256`

Little-endian large integer type 256-bit unsigned integer.


### Type `TransactionView`

The TransactionView objects.

#### Fields

`TransactionView` is a JSON object with the following fields.


*   `type_`: [`U64`](#type-U64) -  64-bit integer that represents the type of the transaction. Axon has evolved to support 3 types of transactions: Legacy is 0x0, Eip2930  is 0x1 and Eip1559  is 0x2.

*   `block_number`: [`U256`](#type-U256)  - QUANTITY - block number where this transaction was in. null when it's pending.

*   `block_hash`: [`H256`](#type-H256)  - DATA, Hex encoded 32 Bytes - hash of the block where this transaction was in. null when its pending.

*   `hash`: [`Hash`](#type-Hash)  - DATA, Hex encoded 32 Bytes - hash of the transaction.

*   `nonce`: [`U256`](#type-U256)  - QUANTITY - the number of transactions made by the sender prior to this one.

*   `from`: [`H160`](#type-H160)  - DATA, Hex encoded 20 Bytes - address of the sender.

*   `to`: [`H160`](#type-H160)  - DATA, Hex encoded 20 Bytes - address of the receiver. null when it's a contract creation transaction.

*   `value`: [`U256`](#type-U256)  -  QUANTITY - value transferred in Wei.

*   `gas`: [`U256`](#type-U256)  - QUANTITY - gas provided by the sender.

*   `gas_price`: [`U256`](#type-U256)  - QUANTITY - gas price provided by the sender in Wei.

*   `max_fee_per_gas`: [`U256`](#type-U256) - QUANTITY - the absolute maximum you are willing to pay per unit of gas to get your transaction included in a block. For brevity and clarity, we will refer to this as the Max Fee.

*   `max_priority_fee_per_gas`: [`U256`](#type-U256)  - QUANTITY - (optional) determined by the user, and is paid directly to miners.

*   `raw`: [`Hex`](#type-Hex)  - The raw is the signed transaction in Recursive Length Prefix (RLP) encoded form

*   `input`: [`Hex`](#type-Hex)  - DATA - The input of the transaction.

*   `public_key`: [`Input`](#type-Input)  - The public key of the transaction sender.

*   `access_list`: [`AccessList`](#type-AccessList)  - A list of addresses and storage keys; these addresses and storage keys are added into the accessed_addresses and accessed_storage_keys global sets.

*   `chain_id`: [`U256`](#type-U256)  - QUANTITY - The id of the chain.

*   `v`: [`U256`](#type-U256)  - QUANTITY - ECDSA recovery id.

*   `r`: [`U256`](#type-U256)  - DATA, 32 Bytes - ECDSA signature r.

*   `s`: [`U256`](#type-U256)  - DATA, 32 Bytes - ECDSA signature s.


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
