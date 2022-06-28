extern crate test;

macro_rules! exec {
    ($func: expr) => {
        futures::executor::block_on(async { $func.await.unwrap() })
    };
}

mod adapter;
mod storage;

use rand::{random, rngs::OsRng};

use common_crypto::{
    Crypto, PrivateKey, Secp256k1Recoverable, Secp256k1RecoverablePrivateKey, Signature,
};
use protocol::types::{
    Block, Eip1559Transaction, ExitReason, ExitSucceed, Hash, Hasher, Header, Proof, Receipt,
    SignatureComponents, SignedTransaction, TransactionAction, UnverifiedTransaction,
};
use protocol::types::{Bytes, UnsignedTransaction};

fn mock_signed_tx() -> SignedTransaction {
    let mut utx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
            nonce:                    Default::default(),
            max_priority_fee_per_gas: Default::default(),
            gas_price:                Default::default(),
            gas_limit:                Default::default(),
            action:                   TransactionAction::Create,
            value:                    Default::default(),
            data:                     Bytes::new(),
            access_list:              vec![],
        }),
        signature: Some(SignatureComponents {
            standard_v: 4,
            r:          Default::default(),
            s:          Default::default(),
        }),
        chain_id:  random::<u64>(),
        hash:      Default::default(),
    }
    .calc_hash();

    let priv_key = Secp256k1RecoverablePrivateKey::generate(&mut OsRng);
    let signature = Secp256k1Recoverable::sign_message(
        utx.signature_hash(true).as_bytes(),
        &priv_key.to_bytes(),
    )
    .unwrap()
    .to_bytes();
    utx.signature = Some(signature.into());

    utx.try_into().unwrap()
}

fn mock_receipt(hash: Hash) -> Receipt {
    Receipt {
        tx_hash:      hash,
        block_number: random::<u64>(),
        block_hash:   Default::default(),
        tx_index:     random::<u32>(),
        state_root:   Default::default(),
        used_gas:     Default::default(),
        logs_bloom:   Default::default(),
        logs:         vec![],
        log_index:    1,
        code_address: None,
        sender:       Default::default(),
        ret:          ExitReason::Succeed(ExitSucceed::Stopped),
        removed:      false,
    }
}

fn mock_block(height: u64, _block_hash: Hash) -> Block {
    let _nonce = Hasher::digest(Bytes::from("XXXX"));
    let header = Header {
        prev_hash:                  Default::default(),
        proposer:                   Default::default(),
        state_root:                 Default::default(),
        transactions_root:          Default::default(),
        signed_txs_hash:            Default::default(),
        receipts_root:              Default::default(),
        log_bloom:                  Default::default(),
        difficulty:                 Default::default(),
        timestamp:                  0,
        number:                     height,
        gas_used:                   Default::default(),
        gas_limit:                  Default::default(),
        extra_data:                 Default::default(),
        mixed_hash:                 None,
        nonce:                      Default::default(),
        base_fee_per_gas:           Default::default(),
        proof:                      Proof::default(),
        last_checkpoint_block_hash: Default::default(),
        chain_id:                   random::<u64>(),
    };

    Block {
        header,
        tx_hashes: vec![],
    }
}

fn mock_proof(block_hash: Hash) -> Proof {
    Proof {
        number: 0,
        round: 0,
        block_hash,
        signature: Default::default(),
        bitmap: Default::default(),
    }
}

fn get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}
