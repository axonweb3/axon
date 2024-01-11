# CHANGELOG
<!--
keep a changelog
Don’t let your friends dump git logs into changelogs.
The format is based on https://keepachangelog.com
-->

## [Unreleased]
* refactor!: change some precompile input and output to tuple by @KaoImin in https://github.com/axonweb3/axon/pull/1642
* refactor: omit `address` field in chain spec parsing by @KaoImin in https://github.com/axonweb3/axon/pull/1641
* feat: rlp encode for VerifyProofPayload by @wenyuanhust in https://github.com/axonweb3/axon/pull/1637
* feat: enable get header precompile by @blckngm in https://github.com/axonweb3/axon/pull/1649
* feat: cli for metadata cell data by @blckngm in https://github.com/axonweb3/axon/pull/1640
* feat: metadata cli parse data by @blckngm in https://github.com/axonweb3/axon/pull/1644
* chore(CI): add axon sync workflow by @Simon-Tl in https://github.com/axonweb3/axon/pull/1636


## v0.3.1-beta

### BUG FIXES
* fix(cli): Fix peer id generation by @samtvlabs in https://github.com/axonweb3/axon/pull/1656


## v0.3.0-beta

### BREAKING CHANGES
* refactor!: remove the limitation of set ckb related info ([\#1517](https://github.com/axonweb3/axon/pull/1517))
* fix(executor)!: set_ckb_related_info transaction is not committed ([\#1576](https://github.com/axonweb3/axon/pull/1576))
* fix!: fix the implementation of Axon Tries ([\#1580](https://github.com/axonweb3/axon/pull/1580))
* refactor!: call reserved system contract address is forbidden ([\#1597](https://github.com/axonweb3/axon/pull/1597))
* refactor!: change many U256 type to U64 ([\#1591](https://github.com/axonweb3/axon/pull/1591))
* fix(mempool)!: check gas limit range ([\#1634](https://github.com/axonweb3/axon/pull/1634))

### FEATURES

* Migrate axon-tools to axon repo ([\#1519](https://github.com/axonweb3/axon/pull/1519), [\#1545](https://github.com/axonweb3/axon/pull/1545))
* Add no-std feature of axon-tools, using use ckb-blst for riscv64 (([\#1532](https://github.com/axonweb3/axon/pull/1532)), [\#1563](https://github.com/axonweb3/axon/pull/1563))
* feat: add `eth_getProof` JSON RPC API ([\#1540](https://github.com/axonweb3/axon/pull/1540), [\#1549](https://github.com/axonweb3/axon/pull/1549), [\#1564](https://github.com/axonweb3/axon/pull/1564), [\#1571](https://github.com/axonweb3/axon/pull/1571))
* feat: add `ckb_blake2b` precompile contract ([\#1555](https://github.com/axonweb3/axon/pull/1555))
* feat: add ckb mbt proof verify precompile contract ([\#1578](https://github.com/axonweb3/axon/pull/1578))
* feat: support stop at specific height ([\#1581](https://github.com/axonweb3/axon/pull/1581))

### BUG FIXES

* Fix value of gas in JSON RPC Transaction should be gas limit ([\#1530](https://github.com/axonweb3/axon/pull/1530))
* fix: check mempool when call `eth_getTransactionByHash` ([\#1526](https://github.com/axonweb3/axon/pull/1526))
* fix: use a same default value for `max_payload_size` ([\#1548](https://github.com/axonweb3/axon/pull/1548))
* fix: rlp decode of `SignedTransction` with interoperation signature ([\#1533](https://github.com/axonweb3/axon/pull/1533))
* fix: get genesis block proposal may panic ([\#1554](https://github.com/axonweb3/axon/pull/1554))
* fix: field `chainId` should be acceptable for `eth_estimateGas` ([\#1601](https://github.com/axonweb3/axon/pull/1601))
* fix: get pending tx count by number ([\#1605](https://github.com/axonweb3/axon/pull/1605))
* fix: duplicated calculation in `eth_estimateGas` ([\#1599](https://github.com/axonweb3/axon/pull/1599), [\#1609](https://github.com/axonweb3/axon/pull/1609))
* fix: gas limit too low error ([\#1625](https://github.com/axonweb3/axon/pull/1625))

### CODE REFACTORS

* Add more details for JSON RPC errors ([\#1495](https://github.com/axonweb3/axon/pull/1495))
* Remove the limitation of set CKB related info in system contract ([\#1517](https://github.com/axonweb3/axon/pull/1517))
* Remove empty crates ([\#1521](https://github.com/axonweb3/axon/pull/1521))
* Enhance readability of output logs and errors ([\#1528](https://github.com/axonweb3/axon/pull/1528))
* Change default db cache size ([\#1531](https://github.com/axonweb3/axon/pull/1531))
* config(blockscan): update the env variables of Axon's explorer ([\#1550](https://github.com/axonweb3/axon/pull/1550))
* Refactor fn is_hardfork_enabled ([\#1538](https://github.com/axonweb3/axon/pull/1538))
* Remove duplicated code to protect ([\#1556](https://github.com/axonweb3/axon/pull/1556))
* Update CkbType.sol and ImageCell.sol for consistent ABI output with json file in ([\#1558](https://github.com/axonweb3/axon/pull/1558), [\#1567](https://github.com/axonweb3/axon/pull/1567))
* refactor: rename Proof.block_hash serde to proposal_hash ([\#1618](https://github.com/axonweb3/axon/pull/1618))
* refactor: forbid call eth_getStorageAt to system contract accounts ([\#1619](https://github.com/axonweb3/axon/pull/1619))
* refactor: change estimate gas calculation logic ([\#1603](https://github.com/axonweb3/axon/pull/1603), [\#1626](https://github.com/axonweb3/axon/pull/1626))
* refactor(cli): update keypair generate command ([\#1621](https://github.com/axonweb3/axon/pull/1621))

### CHORE
* ci: adjust CI after migrating the test projects ([\#1513](https://github.com/axonweb3/axon/pull/1513))
* chore: make blst portable ([\#1520](https://github.com/axonweb3/axon/pull/1520))
* ci: run unit tests in separate processes ([\#1559](https://github.com/axonweb3/axon/pull/1559))
* ci: refactor OpenZeppelin tests and entry_workflow.yml ([\#1610](https://github.com/axonweb3/axon/pull/1610))

## v0.2.0-beta.2

<!--
    Add a summary for the release here.

    If you don't change this message, or if this file is empty, the release
    will not be created. -->
    
This release contains some important bugfixes from the previous 0.2.0-beta.1 version.

### BUG FIXES

- Fix the encode `ConsensusConfig` function
  ([\#1476](https://github.com/axonweb3/axon/pull/1476))
- Fix get metadata by block number
  ([\#1483](https://github.com/axonweb3/axon/pull/1483))
- Fix init EVM config ([\#1484](https://github.com/axonweb3/axon/pull/1484))
- Fix encode and decode of `Proposal` struct
  ([\#1485](https://github.com/axonweb3/axon/pull/1485))

### CODE REFACTORS

- No plain-text private key in configuration file
  ([\#1481](https://github.com/axonweb3/axon/pull/1481))
- Remove default value of `Hex`
  ([\#1482](https://github.com/axonweb3/axon/pull/1482))
- Change the calculation of receipts root to [EIP-2718](https://eips.ethereum.org/EIPS/eip-2718)
  ([\#1486](https://github.com/axonweb3/axon/pull/1486))

### DOCUMENTS

- Add hardfork related APIs document
  ([\#1479](https://github.com/axonweb3/axon/pull/1479))

## v0.2.0-beta.1

<!--
    Add a summary for the release here.

    If you don't change this message, or if this file is empty, the release
    will not be created. -->

Release 0.2.0-beta.1 version.

### BREAKING CHANGES

- Make the calculation of `receipts_root` correct
  ([\#1270](https://github.com/axonweb3/axon/pull/1270))
- Save the receipts of genesis
  ([\#1302](https://github.com/axonweb3/axon/pull/1302))
- Change the position of signature has in interoperation verification
  ([\#1316](https://github.com/axonweb3/axon/pull/1316))
- Add `version` field to block header and proposal
  ([\#1319](https://github.com/axonweb3/axon/pull/1319))
- Read-write separation and merge RocksDB instance
  ([\#1338](https://github.com/axonweb3/axon/pull/1338))
- Remove useless fields in `Header`
  ([\#1339](https://github.com/axonweb3/axon/pull/1339))
- Change the rlp codec of `Hex` ([\#1382](https://github.com/axonweb3/axon/pull/1382))
- Insert metadata directly when initialize chain
  ([\#1454](https://github.com/axonweb3/axon/pull/1454))
- Split bls and secp256k1 private key
  ([\#1471](https://github.com/axonweb3/axon/pull/1471))

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
- Return RPC `null` when no receipt was found
  ([\#1404](https://github.com/axonweb3/axon/pull/1404))
- Fix display of signature v in transaction
  ([\#1431](https://github.com/axonweb3/axon/pull/1431))
- Set CORS to any to accept RPC alls from web-based apps
  ([\#1459](https://github.com/axonweb3/axon/pull/1459))
- Default values of genesis fields should be same as normal blocks
  ([\#1468](https://github.com/axonweb3/axon/pull/1468))

### CODE REFACTOR

- Change serialization for block header `extra_data`
  ([\#1442](https://github.com/axonweb3/axon/pull/1442))
- Remove first transaction in genesis
  ([\#1443](https://github.com/axonweb3/axon/pull/1443)
- Remove metadata precompile contract
  ([\#1449](https://github.com/axonweb3/axon/pull/1449))
- Initialize chain without genesis transaction file
  ([\#1450](https://github.com/axonweb3/axon/pull/1450))
- Initialize chain without any secret key
  ([\#1467](https://github.com/axonweb3/axon/pull/1467))

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
- Make error message display better
  ([\#1405](https://github.com/axonweb3/axon/pull/1405))

### DOCUMENTS

- Add quick start and blocksacn usage
  ([\#1364](https://github.com/axonweb3/axon/pull/1364))
- Fix some typos ([\#1369](https://github.com/axonweb3/axon/pull/1369))
- Update the meaning of genesis transactions
  ([\#1379](https://github.com/axonweb3/axon/pull/1379))
- Update `Getting Started` section in README.md
  ([\#1430](https://github.com/axonweb3/axon/pull/1430))
- Update default dev chain ID
  ([\#1434](https://github.com/axonweb3/axon/pull/1434))

### FEATURES

- Check provided genesis if not first run
  ([\#1278](https://github.com/axonweb3/axon/pull/1278))
- Decode addresses with EIP-55
  ([\#1340](https://github.com/axonweb3/axon/pull/1340))
- Add hardfork storage to metadata system contract
  ([\#1380](https://github.com/axonweb3/axon/pull/1380))
- Split `run` command to `init` and `run`
  ([\#1386](https://github.com/axonweb3/axon/pull/1386))
- Storage batch insert data with sync
  ([\#1389](https://github.com/axonweb3/axon/pull/1389))
- Integrate genesis-generator into axon `init` command
  ([\#1402](https://github.com/axonweb3/axon/pull/1402))
- Add hardfork proposal process
  ([\#1404](https://github.com/axonweb3/axon/pull/1404))
- Support set hardfork in command line
  ([\#1422](https://github.com/axonweb3/axon/pull/1422))
- First hardfork with contract size limit
  ([\#1451](https://github.com/axonweb3/axon/pull/1451))

### PERFORMANCE IMPROVEMENTS

- Reduce memory cost in `modexp` precompile contract
  ([\#1300](https://github.com/axonweb3/axon/pull/1300))

Changelogs before 0.2.0 can be found [here](./CHANGELOG_OLD.md).

