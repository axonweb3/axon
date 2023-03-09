use ckb_types::{
    packed::{self, Header},
    prelude::*,
};
use ethers::{
    abi::{encode, Token},
    types::U256,
};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, H256};

use crate::{
    err,
    precompiles::{axon_precompile_address, PrecompileContract},
    system_contract::ckb_light_client::CkbLightClientHandle,
};

#[derive(Default, Clone)]
pub struct GetHeader;

impl PrecompileContract for GetHeader {
    const ADDRESS: H160 = axon_precompile_address(0x01);
    const MIN_GAS: u64 = 15;

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

        let block_hash = H256::from_slice(input);

        let header = CkbLightClientHandle::default()
            .get_header_by_block_hash(&block_hash)
            .map_err(|_| err!(_, "get header"))?;

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: rlp::encode(&header).to_vec(),
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 3) as u64 + Self::MIN_GAS
    }
}
