pub mod message;

use std::collections::HashMap;
use std::sync::Arc;

use ckb_types::prelude::Unpack;

use common_crypto::{
    BlsPrivateKey, BlsPublicKey, BlsSignature, HashValue, PrivateKey, ToBlsPublicKey,
};
use protocol::tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use protocol::traits::{Context, CrossAdapter, MessageTarget};
use protocol::types::{Bytes, Hasher, Requests, ValidatorExtend, H160, H256};
use protocol::{codec::ProtocolCodec, tokio, ProtocolResult};

use crate::error::CrossChainError;
use crate::task::message::{
    CrossChainMessage, CrossChainSignature, TxViewWrapper, END_GOSSIP_BUILD_CKB_TX,
    END_GOSSIP_CKB_TX_SIGNATURE,
};

pub struct RequestCkbTask<Adapter> {
    address:        H160,
    private_key:    BlsPrivateKey,
    public_key:     BlsPublicKey,
    validators:     Vec<ValidatorExtend>,
    validate_power: bool,
    req_records:    ReqRecords,

    reqs_rx: UnboundedReceiver<Requests>,
    net_rx:  UnboundedReceiver<CrossChainMessage>,

    adapter: Arc<Adapter>,
}

impl<Adapter> RequestCkbTask<Adapter>
where
    Adapter: CrossAdapter + 'static,
{
    pub async fn new(
        addr: H160,
        private_key: &[u8],
        reqs_rx: UnboundedReceiver<Requests>,
        adapter: Arc<Adapter>,
    ) -> (Self, UnboundedSender<CrossChainMessage>) {
        let (net_tx, net_rx) = unbounded_channel();
        let private_key = BlsPrivateKey::try_from(private_key).expect("Invalid bls private key");

        let task = RequestCkbTask {
            address: addr,
            private_key: private_key.clone(),
            public_key: private_key.pub_key(&String::new()),
            validators: vec![],
            validate_power: false,
            req_records: Default::default(),

            reqs_rx,
            net_rx,

            adapter,
        };

        (task, net_tx)
    }

    async fn is_leader(&mut self, reqs: &Requests) -> bool {
        self.update_metadata().await;

        if !self.validate_power {
            return false;
        }

        let offset = reqs.0.len() % self.validators.len();
        self.validators[offset].address == self.address
    }

    fn calc_leader(&self, hash: &H256) -> H160 {
        let reqs = self.req_records.get_req(hash).unwrap();
        self.validators[reqs.0.len() % self.validators.len()].address
    }

    async fn update_metadata(&mut self) {
        let mut validators = self
            .adapter
            .current_metadata(Context::new())
            .await
            .verifier_list;
        validators.sort();
        self.validators = validators;
        self.update_validate_power().await;
        self.req_records
            .update_threshold((self.validators.len() / 3) * 2 + 1)
    }

    async fn update_validate_power(&mut self) {
        self.validate_power = self.validators.iter().any(|ve| ve.address == self.address);
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some(reqs) = self.reqs_rx.recv() => {
                    log::info!("[cross-chain]: receive cross to CKB requests, len {}", reqs.0.len());

                    if let Err(e) = self.handle_reqs(reqs).await {
                        log::error!("[cross-chain]: handle requests error {:?}", e);
                    }
                }
                Some(msg) = self.net_rx.recv() => {
                    log::info!("[cross-chain]: receive network message {:?}", msg);

                    if let Err(e) = self.handle_msgs(msg).await {
                        log::error!("[cross-chain]: handle message error {:?}", e);
                    }
                }
            }
        }
    }

    async fn handle_msgs(&mut self, msg: CrossChainMessage) -> ProtocolResult<()> {
        if !self.validate_power {
            return Ok(());
        }

        match msg {
            CrossChainMessage::TxView(tx_view_wrapper) => {
                if let Ok(sig) = self.verify_tx_wrapper(&tx_view_wrapper) {
                    self.adapter
                        .transmit(
                            Context::new(),
                            CrossChainMessage::Signature(sig).encode().unwrap().to_vec(),
                            END_GOSSIP_CKB_TX_SIGNATURE,
                            MessageTarget::Specified(Bytes::from(
                                self.calc_leader(&tx_view_wrapper.req_hash)
                                    .to_fixed_bytes()
                                    .to_vec(),
                            )),
                        )
                        .await?;
                }
            }

            CrossChainMessage::Signature(sig) => {
                if let Some((c_sig, pks)) =
                    self.req_records.insert_vote(&sig.req_hash, (&sig).into())
                {
                    let _tx =
                        self.adapter
                            .build_to_ckb_tx(Context::new(), sig.tx_hash, &c_sig, &pks);
                }
            }
        }

        Ok(())
    }

    async fn handle_reqs(&mut self, reqs: Requests) -> ProtocolResult<()> {
        if !self.is_leader(&reqs).await {
            log::warn!("[cross-client]: do not have power");
            return Ok(());
        }

        let hash = self.req_records.add_req(reqs.clone());

        log::info!("[cross-chain]: cross to CKB request {:?}", reqs.0);

        let ctx = Context::new();
        let tx_view = self.adapter.calc_to_ckb_tx(ctx.clone(), &reqs.0).await?;
        self.req_records
            .update_tx_hash(&hash, H256(tx_view.hash().unpack().0));

        let tx_wrapper = TxViewWrapper::new(hash, tx_view);

        if self.validators.len() == 1 {
            let sig = self.verify_tx_wrapper(&tx_wrapper)?;
            let comb_sig =
                BlsSignature::combine(vec![(sig.signature.clone(), sig.bls_pubkey.clone())])
                    .map_err(|e| CrossChainError::Crypto(e.to_string()))?;

            let intact_tx =
                self.adapter
                    .build_to_ckb_tx(ctx.clone(), sig.tx_hash, &comb_sig, &[sig.bls_pubkey])?;

            return self
                .adapter
                .send_ckb_tx(ctx.clone(), intact_tx.into())
                .await;
        }

        self.adapter
            .transmit(
                ctx.clone(),
                CrossChainMessage::TxView(tx_wrapper)
                    .encode()
                    .unwrap()
                    .to_vec(),
                END_GOSSIP_BUILD_CKB_TX,
                MessageTarget::Broadcast,
            )
            .await
    }

    fn verify_tx_wrapper(&self, tx: &TxViewWrapper) -> ProtocolResult<CrossChainSignature> {
        // Todo: add verify process
        let sig = self
            .private_key
            .sign_message(&HashValue::from_bytes_unchecked(
                tx.tx_view.hash().unpack().0,
            ));

        Ok(CrossChainSignature {
            req_hash:   tx.req_hash,
            tx_hash:    H256(tx.tx_view.hash().unpack().0),
            bls_pubkey: self.public_key.clone(),
            signature:  sig,
        })
    }
}

#[derive(Default, Clone, Debug)]
struct ReqRecords(HashMap<H256, ReqBucket>, usize);

impl ReqRecords {
    fn new(threshold: usize) -> Self {
        ReqRecords(HashMap::new(), threshold)
    }

    fn get_req(&self, hash: &H256) -> Option<Requests> {
        self.0.get(hash).map(|b| b.reqs.clone())
    }

    fn add_req(&mut self, req: Requests) -> H256 {
        let hash = Hasher::digest(&req.encode().unwrap());
        self.0.entry(hash).or_insert_with(|| ReqBucket::new(req));
        hash
    }

    fn update_tx_hash(&mut self, req_hash: &H256, tx_hash: H256) {
        if let Some(b) = self.0.get_mut(req_hash) {
            b.set_tx_hash(tx_hash);
        }
    }

    fn update_threshold(&mut self, threshold: usize) {
        self.1 = threshold;
    }

    fn insert_vote(
        &mut self,
        req_hash: &H256,
        vote: Vote,
    ) -> Option<(BlsSignature, Vec<BlsPublicKey>)> {
        if let Some(b) = self.0.get_mut(req_hash) {
            b.add_vote(vote);

            if b.votes.len() == self.1 {
                let mut sig_pks = Vec::with_capacity(self.1);
                let mut pks = Vec::with_capacity(self.1);

                for v in b.votes.iter() {
                    sig_pks.push((v.signature.clone(), v.pubkey.clone()));
                    pks.push(v.pubkey.clone());
                }

                return Some((BlsSignature::combine(sig_pks).unwrap(), pks));
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
struct ReqBucket {
    reqs:    Requests,
    tx_hash: H256,
    votes:   Vec<Vote>,
}

impl ReqBucket {
    fn new(reqs: Requests) -> Self {
        ReqBucket {
            reqs,
            tx_hash: H256::default(),
            votes: Vec::new(),
        }
    }

    fn set_tx_hash(&mut self, tx_hash: H256) {
        self.tx_hash = tx_hash;
    }

    fn add_vote(&mut self, vote: Vote) {
        self.votes.push(vote);
    }
}

#[derive(Clone, Debug)]
struct Vote {
    pubkey:    BlsPublicKey,
    signature: BlsSignature,
}

impl From<&CrossChainSignature> for Vote {
    fn from(signature: &CrossChainSignature) -> Self {
        Vote {
            pubkey:    signature.bls_pubkey.clone(),
            signature: signature.signature.clone(),
        }
    }
}
