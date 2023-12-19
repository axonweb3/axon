use ckb_types::{packed, prelude::Entity};
use ethers::abi::{AbiDecode, AbiEncode};
use ethers::contract::{EthAbiCodec, EthAbiType};
use ethers::core::types::Bytes as EthBytes;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, H256};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::system_contract::image_cell::{image_cell_abi, CellKey, ImageCellReader};
use crate::{err, CURRENT_HEADER_CELL_ROOT};

#[derive(Default, Clone)]
pub struct GetCell;

impl PrecompileContract for GetCell {
    const ADDRESS: H160 = axon_precompile_address(0x03);
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

        let (tx_hash, index) = parse_input(input)?;

        let root = CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow());
        let cell_opt = ImageCellReader
            .get_cell(root, &CellKey { tx_hash, index })
            .map_err(|_| err!(_, "get cell"))?
            .map(|c| Cell {
                cell_output:     packed::CellOutput::new_unchecked(c.cell_output).into(),
                cell_data:       c.cell_data.into(),
                is_consumed:     c.consumed_number.is_some(),
                created_number:  c.created_number,
                consumed_number: c.consumed_number.unwrap_or(0),
            });

        if cell_opt.is_none() {
            return err!("get cell return None");
        }

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      AbiEncode::encode((cell_opt.unwrap(),)),
            },
            gas,
        ))
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}

fn parse_input(input: &[u8]) -> Result<(H256, u32), PrecompileFailure> {
    let out_point = <image_cell_abi::OutPoint as AbiDecode>::decode(input)
        .map_err(|_| err!(_, "decode input"))?;

    Ok((H256(out_point.tx_hash), out_point.index))
}

#[derive(EthAbiType, EthAbiCodec, Default, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub cell_output:     image_cell_abi::CellOutput,
    pub cell_data:       EthBytes,
    pub is_consumed:     bool,
    pub created_number:  u64,
    pub consumed_number: u64,
}
