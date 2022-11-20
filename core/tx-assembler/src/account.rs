use ckb_types::{
    core::Capacity,
    packed::{CellOutput, Script},
    prelude::Entity,
};
use hasher::{Hasher as KeccakHasher, HasherKeccak};
use protocol::types::{Account, H160, NIL_DATA, RLP_NULL, U256, H256};

lazy_static::lazy_static! {
    static ref HASHER_INST: HasherKeccak = HasherKeccak::new();
}

#[warn(dead_code)]
pub struct EthereumAccount {
    _lock_script: Script,
    _eth_address: H160,
    _eth_account: Account,
    _cell:        CellOutput,
}

impl EthereumAccount {
    #[allow(unused)]
    pub fn new(lock_script: Script) -> Self {
        let cell = CellOutput::new_builder()
            .lock(lock_script.clone())
            .build_exact_capacity(Capacity::zero())
            .unwrap();
        let eth_account = Account {
            nonce:        U256::zero(),
            balance:      U256::zero(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        };
        let raw_address = H256::from_slice(&HASHER_INST.digest(&lock_script.as_bytes()));
        let eth_address = H160::from(raw_address);
        EthereumAccount {
            _lock_script: lock_script,
            _eth_address: eth_address,
            _eth_account: eth_account,
            _cell: cell,
        }
    }

    #[allow(unused)]
    pub fn get_account() {}
}

#[cfg(test)]
mod tests {

    use crate::util;

    use super::*;

    #[test]
    fn test_gen_account() {
        let mock_arg = protocol::types::Hash::random();
        let account = EthereumAccount::new(util::build_acs_lock_script(mock_arg));
        println!("{:?}", account._eth_address);
        // 0xdbcbe8e1f26ad869c5ff455bca16ec4349e8c6b2
    }

}
