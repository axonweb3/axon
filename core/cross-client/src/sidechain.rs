use std::sync::Arc;

use ckb_jsonrpc_types::OutputsValidator;

use common_crypto::Secp256k1RecoverablePrivateKey;
use protocol::codec::ProtocolCodec;
use protocol::traits::{CkbClient, Context};
use protocol::types::{Block, Hash, Identity, Proof, Proposal, SubmitCheckpointPayload, H160};

pub struct SidechainTask {
    priv_key:             Secp256k1RecoverablePrivateKey,
    node_address:         H160,
    admin_address:        H160,
    selection_lock_hash:  Hash,
    checkpoint_type_hash: Hash,
}

impl SidechainTask {
    pub fn new(
        priv_key: Secp256k1RecoverablePrivateKey,
        node_address: H160,
        admin_address: H160,
        selection_lock_hash: Hash,
        checkpoint_type_hash: Hash,
    ) -> Self {
        SidechainTask {
            priv_key,
            node_address,
            admin_address,
            selection_lock_hash,
            checkpoint_type_hash,
        }
    }

    pub async fn run<C: CkbClient + 'static>(self, client: Arc<C>, block: Block, proof: Proof) {
        let number = block.header.number;
        let mut proposal = Proposal::from(block).encode().unwrap().to_vec();
        let mut proof = proof.encode().unwrap().to_vec();
        proposal.append(&mut proof);

        let payload = SubmitCheckpointPayload {
            node_id:              Identity::new(0, self.node_address.0.to_vec()),
            admin_id:             Identity::new(0, self.admin_address.0.to_vec()),
            period_number:        number,
            checkpoint:           proposal.into(),
            selection_lock_hash:  self.selection_lock_hash.0.into(),
            checkpoint_type_hash: self.checkpoint_type_hash.0.into(),
        };

        match client
            .build_submit_checkpoint_transaction(Context::new(), payload)
            .await
        {
            Ok(respond) => {
                let tx = respond.sign(&self.priv_key);
                match client
                    .send_transaction(Context::new(), &tx, Some(OutputsValidator::Passthrough))
                    .await
                {
                    Ok(tx_hash) => log::info!("set_checkpoint send tx hash: {}", tx_hash),
                    Err(e) => {
                        log::info!("set_checkpoint send tx error: {}", e);
                    }
                }
            }
            Err(e) => {
                log::info!("build_submit_checkpoint_transaction error: {}", e);
            }
        }
    }
}
