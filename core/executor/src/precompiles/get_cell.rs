use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use crate::precompiles::{eip_precompile_address, PrecompileContract};
use crate::system_contract::image_cell::CellKey;
use crate::{err, system_contract::image_cell::ImageCellContract};

#[derive(Default, Clone)]
pub struct GetCell;

impl PrecompileContract for GetCell {
    const ADDRESS: H160 = eip_precompile_address(0xf0);
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

        let ret = ImageCellContract::default()
            .get_cell(&CellKey::decode(input).map_err(|_| err!(_, "decode cell key"))?)
            .map_err(|_| err!(_, "get cell"))?;

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      rlp::encode(&ret).to_vec(),
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 3) as u64 + Self::MIN_GAS
    }
}
