use std::sync::Arc;

use evm::{ExitReason, ExitSucceed};
use hashbrown::HashMap;
use revm::{AccountInfo, Bytecode, Database, DatabaseCommit};

use common_merkle::Merkle;
use core_executor::{code_address, MPTTrie};
use protocol::traits::{Context, Storage};
use protocol::types::{
    Account, Address, Bytes, ExecResp, ExecutorContext, Hasher, Proposal, SignedTransaction,
    TransactionAction, TxResp, H160, H256, NIL_DATA, RLP_NULL, U256,
};
use protocol::{codec::ProtocolCodec, ProtocolError};

lazy_static::lazy_static! {
    static ref DISTRIBUTE_ADDRESS: Address = Address::from_hex("0x35e70c3f5a794a77efc2ec5ba964bffcc7fd2c0a").unwrap();
}

macro_rules! blocking_async {
    ($self_: ident, $adapter: ident, $method: ident$ (, $args: expr)*) => {{
        let rt = protocol::tokio::runtime::Handle::current();
        let adapter = Arc::clone(&$self_.$adapter);

        protocol::tokio::task::block_in_place(move || {
            rt.block_on(adapter.$method( $($args,)* )).unwrap()
        })
    }};
}

pub struct RevmAdapter<S, DB: cita_trie::DB> {
    exec_ctx: ExecutorContext,
    trie:     MPTTrie<DB>,
    storage:  Arc<S>,
    db:       Arc<DB>,
}

impl<S, DB> Database for RevmAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    type Error = ProtocolError;

    fn basic(&mut self, address: H160) -> Result<Option<AccountInfo>, Self::Error> {
        let raw = self.trie.get(address.as_bytes())?;
        if raw.is_none() {
            return Ok(None);
        }

        let raw = raw.unwrap();
        Ok(Some(Account::decode(raw).map(|a| AccountInfo {
            balance:   a.balance,
            nonce:     a.nonce.as_u64(),
            code_hash: a.code_hash,
            code:      None,
        })?))
    }

    fn code_by_hash(&mut self, code_hash: H256) -> Result<Bytecode, Self::Error> {
        Ok(if code_hash == NIL_DATA {
            Bytecode::new()
        } else {
            match blocking_async!(self, storage, get_code_by_hash, Context::new(), &code_hash) {
                Some(bytes) => Bytecode::new_raw(bytes),
                None => Bytecode::new(),
            }
        })
    }

    fn storage(&mut self, address: H160, index: U256) -> Result<U256, Self::Error> {
        let raw = self.trie.get(address.as_bytes())?;
        if raw.is_none() {
            return Ok(U256::default());
        }

        let raw = raw.unwrap();
        let account = Account::decode(raw)?;
        let storage_root = account.storage_root;
        if storage_root == RLP_NULL {
            Ok(U256::default())
        } else {
            MPTTrie::from_root(storage_root, Arc::clone(&self.db)).map(|storage| {
                match storage.get(u256_to_u8_slice(&index)) {
                    Ok(Some(bytes)) => match u8_slice_to_u256(bytes.as_ref()) {
                        Some(res) => res,
                        None => U256::default(),
                    },
                    _ => U256::default(),
                }
            })
        }
    }

    fn block_hash(&mut self, number: U256) -> Result<H256, Self::Error> {
        let current_number = self.exec_ctx.block_number;
        if number > current_number {
            return Ok(H256::default());
        }

        let number = number.as_u64();
        let res = blocking_async!(self, storage, get_block, Context::new(), number)
            .map(|b| Proposal::from(&b).hash())
            .unwrap_or_default();

        Ok(res)
    }
}

impl<S, DB> DatabaseCommit for RevmAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    fn commit(&mut self, changes: HashMap<H160, revm::Account>) {
        changes.into_iter().for_each(|(addr, change)| {
            if change.is_empty() {
                let _ = self.trie.remove(addr.as_bytes());
            }
            let old_account = match self.trie.get(addr.as_bytes()) {
                Ok(Some(raw)) => Account::decode(raw).unwrap(),
                _ => Account {
                    nonce:        U256::zero(),
                    balance:      U256::zero(),
                    storage_root: RLP_NULL,
                    code_hash:    NIL_DATA,
                },
            };

            let storage_root = old_account.storage_root;
            let mut storage_trie = if storage_root == RLP_NULL {
                MPTTrie::new(Arc::clone(&self.db))
            } else {
                MPTTrie::from_root(storage_root, Arc::clone(&self.db)).unwrap()
            };
            change.storage.into_iter().for_each(|(k, v)| {
                let _ =
                    storage_trie.insert(u256_to_u8_slice(&k), u256_to_u8_slice(&v.present_value()));
            });

            let code_hash = if let Some(code) = change.info.code {
                let code_hash = change.info.code_hash;
                if code_hash != old_account.code_hash {
                    blocking_async!(
                        self,
                        storage,
                        insert_code,
                        Context::new(),
                        addr.into(),
                        code_hash,
                        code.bytes().clone()
                    );
                }
                code_hash
            } else {
                NIL_DATA
            };

            let new_account = Account {
                nonce: U256::from(change.info.nonce),
                balance: change.info.balance,
                storage_root: storage_trie.commit().unwrap(),
                code_hash,
            };

            let account_bytes = new_account.encode().unwrap();
            self.trie
                .insert(addr.as_bytes(), account_bytes.as_ref())
                .unwrap();
        });
    }
}

impl<S, DB> RevmAdapter<S, DB>
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    pub fn init_mpt(&mut self, account: Account, addr: Address) {
        let mut mpt = MPTTrie::new(Arc::clone(&self.db));
        let distribute_account = account;

        mpt.insert(
            addr.as_slice(),
            distribute_account.encode().unwrap().as_ref(),
        )
        .unwrap();

        mpt.commit().unwrap();
        self.trie = mpt;
    }

    pub fn new(storage: S, db: DB, exec_ctx: ExecutorContext) -> Self {
        let db = Arc::new(db);
        RevmAdapter {
            exec_ctx,
            trie: MPTTrie::new(Arc::clone(&db)),
            storage: Arc::new(storage),
            db,
        }
    }
}

#[inline]
fn u256_to_u8_slice(index: &U256) -> &[u8] {
    let u64_slice = index.as_ref();
    let result: &[u8] = bytemuck::cast_slice(u64_slice);
    result
}

#[inline]
fn u8_slice_to_u256(bytes: &[u8]) -> Option<U256> {
    let u64_slice: &[u64] = bytemuck::cast_slice(bytes);
    let array: &[u64; 4] = u64_slice.try_into().expect("incorrect length");
    Some(U256(array.to_owned()))
}

fn set_revm<T: Database>(
    evm: &mut revm::EVM<T>,
    gas_limit: u64,
    from: Option<H160>,
    to: Option<H160>,
    value: U256,
    data: Vec<u8>,
) {
    evm.env.tx.gas_limit = gas_limit;
    if let Some(caller) = from {
        evm.env.tx.caller = caller;
    }
    evm.env.tx.data = Bytes::from(data);
    evm.env.tx.value = value;
    evm.env.tx.transact_to = if let Some(to) = to {
        revm::TransactTo::Call(to)
    } else {
        revm::TransactTo::Create(revm::CreateScheme::Create)
    };
}

#[allow(dead_code)]
pub fn revm_call<T: Database>(
    db: T,
    gas_limit: u64,
    from: Option<H160>,
    to: Option<H160>,
    value: U256,
    data: Vec<u8>,
) -> TxResp {
    let mut evm = revm::EVM::<T>::new();
    evm.database(db);
    set_revm(&mut evm, gas_limit, from, to, value, data);
    let (res, _state) = evm.transact();
    TxResp {
        // todo
        exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
        ret:          match res.out {
            revm::TransactOut::None => vec![],
            revm::TransactOut::Call(ret) => ret.to_vec(),
            revm::TransactOut::Create(ret, _) => ret.to_vec(),
        },
        gas_used:     res.gas_used,
        remain_gas:   gas_limit - res.gas_used,
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

pub fn revm_exec<S, DB>(
    evm: &mut revm::EVM<RevmAdapter<S, DB>>,
    txs: Vec<SignedTransaction>,
) -> ExecResp
where
    S: Storage + 'static,
    DB: cita_trie::DB + 'static,
{
    let txs_len = txs.len();
    let mut tx_outputs = Vec::with_capacity(txs_len);
    let mut hashes = Vec::with_capacity(txs_len);
    let mut total_gas_used = 0u64;

    txs.into_iter().for_each(|tx| {
        let old_nonce = evm
            .db
            .as_mut()
            .unwrap()
            .basic(tx.sender)
            .unwrap()
            .unwrap_or_default()
            .nonce;
        set_revm(
            evm,
            tx.transaction.unsigned.gas_limit().as_u64(),
            Some(tx.sender),
            tx.transaction.unsigned.to(),
            *tx.transaction.unsigned.value(),
            tx.transaction.unsigned.data().to_vec(),
        );
        let res = evm.transact_commit();
        let ret = match res.out {
            revm::TransactOut::None => vec![],
            revm::TransactOut::Call(bytes) => bytes.to_vec(),
            revm::TransactOut::Create(bytes, _) => bytes.to_vec(),
        };
        hashes.push(Hasher::digest(&ret));
        total_gas_used += res.gas_used;

        let code_address = if tx.transaction.unsigned.action() == &TransactionAction::Create {
            Some(code_address(&tx.sender, &U256::from(old_nonce)))
        } else {
            None
        };

        let resp = TxResp {
            exit_reason: evm::ExitReason::Succeed(ExitSucceed::Returned),
            ret,
            gas_used: res.gas_used,
            remain_gas: tx.transaction.unsigned.gas_limit().as_u64() - res.gas_used,
            // todo
            logs: vec![],
            code_address,
            removed: false,
        };
        tx_outputs.push(resp);
    });

    ExecResp {
        state_root:   evm.db().unwrap().trie.commit().unwrap(),
        receipt_root: Merkle::from_hashes(hashes)
            .get_root_hash()
            .unwrap_or_default(),
        gas_used:     total_gas_used,
        tx_resp:      tx_outputs,
    }
}
