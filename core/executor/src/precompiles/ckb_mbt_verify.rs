use ckb_types::packed;
use ckb_types::prelude::Pack;
use ckb_types::utilities::{merkle_root, MerkleProof};
use ethers::abi::AbiDecode;
use ethers::contract::{EthAbiCodec, EthAbiType};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use crate::err;
use crate::precompiles::{axon_precompile_address, PrecompileContract};

#[derive(EthAbiCodec, EthAbiType, Clone, Debug, PartialEq, Eq)]
pub struct VerifyProofPayload {
    pub transactions_root:     [u8; 32],
    pub witnesses_root:        [u8; 32],
    pub raw_transactions_root: [u8; 32],
    pub indices:               Vec<u32>,
    pub lemmas:                Vec<[u8; 32]>,
    pub leaves:                Vec<[u8; 32]>,
}

#[derive(Default, Clone)]
pub struct CMBTVerify;

impl PrecompileContract for CMBTVerify {
    const ADDRESS: H160 = axon_precompile_address(0x07);
    const MIN_GAS: u64 = 56000;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if gas > limit {
                return err!();
            }
        }

        let payload = parse_input(input)?;
        inner_verify_proof(payload)?;

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      vec![true.into()],
            },
            gas,
        ))
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}

fn parse_input(input: &[u8]) -> Result<VerifyProofPayload, PrecompileFailure> {
    <VerifyProofPayload as AbiDecode>::decode(input).map_err(|_| err!(_, "decode input"))
}

fn inner_verify_proof(payload: VerifyProofPayload) -> Result<(), PrecompileFailure> {
    // Firstly, verify the transactions_root is consist of the raw_transactions_root
    // and witnesses_root
    let transactions_root: packed::Byte32 = payload.transactions_root.pack();
    let raw_transactions_root: packed::Byte32 = payload.raw_transactions_root.pack();
    let witnesses_root: packed::Byte32 = payload.witnesses_root.pack();

    if merkle_root(&[raw_transactions_root.clone(), witnesses_root]) != transactions_root {
        return Err(err!(_, "verify transactions_root fail"));
    }

    // Then, verify the given indices and lemmas can prove the leaves contains in
    // the raw_transactions_root
    let lemmas = payload.lemmas.iter().map(|l| l.pack()).collect::<Vec<_>>();
    let leaves = payload.leaves.iter().map(|l| l.pack()).collect::<Vec<_>>();

    if MerkleProof::new(payload.indices, lemmas).verify(&raw_transactions_root, &leaves) {
        return Ok(());
    }

    Err(err!(_, "verify raw transactions root failed"))
}
