## Unreleased

### BREAKING CHANGES

- Make the calculation of `receipts_root` correct
  ([\#1270](https://github.com/axonweb3/axon/pull/1270))
- Save the receipts of genesis
  ([\#1302](https://github.com/axonweb3/axon/pull/1302))
- Change the position of signature has in interoperation verification
  ([\#1316](https://github.com/axonweb3/axon/pull/1316))
- Add `version` field to block header and proposal
  ([\#1319](https://github.com/axonweb3/axon/pull/1319))
- Change the rlp codec of `Hex` ([\#1338](https://github.com/axonweb3/axon/pull/1338))
- Read-write separation and merge RocksDB instance
  ([\#1338](https://github.com/axonweb3/axon/pull/1338))
- Remove useless fields in `Header`
  ([\#1339](https://github.com/axonweb3/axon/pull/1339))

### BUG FIXES

- Always require CLI sub-command
  ([\#1269](https://github.com/axonweb3/axon/pull/1269))
- Fix `eth_subscribe` method name ([\#1281](https://github.com/axonweb3/axon/pull/1281))
- `modexp` precompile contract may overflow
  ([\#1299](https://github.com/axonweb3/axon/pull/1299))
- No default values when unexpected errors occurred
  ([\#1303](https://github.com/axonweb3/axon/pull/1303))
- Gas limit conversion error
  ([\#1344](https://github.com/axonweb3/axon/pull/1344))
- Prometheus can not be disabled
  ([\#1348](https://github.com/axonweb3/axon/pull/1348))
- Make genesis hash correct
  ([\#1353](https://github.com/axonweb3/axon/pull/1353))
- Use a same trie for the same chain
  ([\#1370](https://github.com/axonweb3/axon/pull/1370))
- The  deserialize of  ([\#1396](https://github.com/axonweb3/axon/pull/1396))
- `genesis-generator` is nondeterministic ([\#1400](https://github.com/axonweb3/axon/pull/1400))

### CODE REFACTORS

- Use thread local instead of global variable
  ([\#1280](https://github.com/axonweb3/axon/pull/1280))
- Change the `call_ckb_vm` and `verify_by_ckb` precompile argument payload
  ([\#1285](https://github.com/axonweb3/axon/pull/1285))
- Get client version from crate version and git commit ID
  ([\#1309](https://github.com/axonweb3/axon/pull/1309))
- Use constant values as default values
  ([\#1320](https://github.com/axonweb3/axon/pull/1320))
- Use derive procedural macro to parse command line arguments
  ([\#1326](https://github.com/axonweb3/axon/pull/1326))
- Change config to single struct
  ([\#1328](https://github.com/axonweb3/axon/pull/1328))
- Split config and genesis to client config and chain spec
  ([\#1332](https://github.com/axonweb3/axon/pull/1332))
- Remove deprecated config ([\#1333](https://github.com/axonweb3/axon/pull/1333))
- Split the all-in-one file of `core-run`
  ([\#1349](https://github.com/axonweb3/axon/pull/1349))
- Make chain id immutable ([\#1351](https://github.com/axonweb3/axon/pull/1351))
- Remove useless tests of solidity contract
  ([\#1355](https://github.com/axonweb3/axon/pull/1355))
- Implement `Trie` for `MPTTrie` ([\#1363](https://github.com/axonweb3/axon/pull/1363))
- Slightly improve security for secret keys
  ([\#1367](https://github.com/axonweb3/axon/pull/1367))
- Change the memory layout of `Hex`
  ([\#1371](https://github.com/axonweb3/axon/pull/1371))

### DOCUMENTS

- Add quick start and blocksacn usage
  ([\#1364](https://github.com/axonweb3/axon/pull/1364))
- Fix some typos ([\#1369](https://github.com/axonweb3/axon/pull/1369))
- Update the meaning of genesis transactions
  ([\#1379](https://github.com/axonweb3/axon/pull/1379))

### FEATURES

- Check provided genesis if not first run
  ([\#1278](https://github.com/axonweb3/axon/pull/1278))
- Decode addresses with EIP-55
  ([\#1340](https://github.com/axonweb3/axon/pull/1340))
- Add hardfork storage to metadata system contract
  ([\#1380](https://github.com/axonweb3/axon/pull/1380))
- Storage batch insert data with sync
  ([\#1389](https://github.com/axonweb3/axon/pull/1389))

### PERFORMANCE IMPROVEMENTS

- Reduce memory cost in `modexp` precompile contract
  ([\#1300](https://github.com/axonweb3/axon/pull/1300))

Changelogs before 0.2.0 can be found [here](./CHANGELOG_OLD.md).
