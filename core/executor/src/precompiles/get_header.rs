use ethers::abi::AbiDecode;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, H256};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::{err, system_contract::CkbLightClientContract};

#[derive(Default, Clone)]
pub struct GetHeader;

impl PrecompileContract for GetHeader {
    const ADDRESS: H160 = axon_precompile_address(0x02);
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

        let block_hash =
            H256(<[u8; 32] as AbiDecode>::decode(input).map_err(|_| err!(_, "decode input"))?);

        let raw = CkbLightClientContract::default()
            .get_raw(&block_hash.0)
            .map_err(|_| err!(_, "get header"))?;

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      raw.unwrap_or_default(),
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 3) as u64 + Self::MIN_GAS
    }
}
