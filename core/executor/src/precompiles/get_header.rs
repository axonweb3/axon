use ethers::abi::AbiDecode;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, H256};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::{err, system_contract::ckb_light_client::CkbHeaderReader, CURRENT_HEADER_CELL_ROOT};

#[derive(Default, Clone)]
pub struct GetHeader;

impl PrecompileContract for GetHeader {
    const ADDRESS: H160 = axon_precompile_address(0x02);
    const MIN_GAS: u64 = 42000;

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

        let root = CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow());
        let header_opt = CkbHeaderReader
            .get_raw(root, &block_hash.0)
            .map_err(|_| err!(_, "get header"))?;

        if header_opt.is_none() {
            return err!("get header return None");
        }

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      header_opt.unwrap(),
            },
            gas,
        ))
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}
