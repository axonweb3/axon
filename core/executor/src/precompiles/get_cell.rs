use ckb_types::{packed, prelude::*};
use ethers::{
    abi::{encode, parse_abi, FunctionExt, Token},
    types::U256,
};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, H256};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::system_contract::image_cell::{CellInfo, CellKey};
use crate::{err, system_contract::image_cell::ImageCellContract};

#[derive(Default, Clone)]
pub struct GetCell;

impl PrecompileContract for GetCell {
    const ADDRESS: H160 = axon_precompile_address(0xf0);
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

        let (tx_hash, index) = parse_input(input)?;

        let cell = ImageCellContract::default()
            .get_cell(&CellKey { tx_hash, index })
            .map_err(|_| err!(_, "get cell"))?;

        let output = if let Some(cell) = cell {
            encode_cell(cell)
        } else {
            encode_default()
        };

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output,
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 3) as u64 + Self::MIN_GAS
    }
}

fn parse_input(input: &[u8]) -> Result<(H256, u32), PrecompileFailure> {
    let contract = parse_abi(&[
        "function getCell(bytes32 txHash, uint32 index) external returns (tuple(bool exists, bool hasTypeScript, bool hasConsumedNumber, uint64 createdNumber, uint64 consumedNumber, uint64 capacity, uint8 lockHashType, uint8 typeHashType, bytes lockCodeHash, bytes typeCodeHash, bytes lockArgs, bytes typeArgs, bytes data) memory)",
    ]).map_err(|_| err!(_, "invalid abi"))?;

    let function = contract
        .functions()
        .find(|fun| fun.selector() == input[0..4])
        .ok_or(err!(_, "unknown selector"))?;

    let func_input = function
        .decode_input(&input[4..])
        .map_err(|_| err!(_, "invalid function name"))?;

    match &func_input[..] {
        [Token::FixedBytes(tx_hash), Token::Uint(index)] => {
            Ok((H256::from_slice(&tx_hash[..]), index.as_u32()))
        }
        _ => Err(err!(_, "invalid input")),
    }
}

fn encode_cell(cell: CellInfo) -> Vec<u8> {
    let cell_output = packed::CellOutput::new_unchecked(cell.cell_output);

    let lock = cell_output.lock();
    let lock_hash_type: u8 = lock.hash_type().into();

    let type_ = cell_output.type_();
    let (type_script_exists, type_hash_type, type_code_hash, type_args) =
        if let Some(type_) = type_.to_opt() {
            let hash_type: u8 = type_.hash_type().into();
            (
                true,
                hash_type,
                type_.code_hash().raw_data().to_vec(),
                type_.args().raw_data().to_vec(),
            )
        } else {
            (false, 0, Vec::with_capacity(32), Vec::new())
        };

    let capacity: u64 = cell_output.capacity().unpack();

    encode(&[Token::Tuple(vec![
        Token::Bool(true),
        Token::Bool(type_script_exists),
        Token::Bool(cell.consumed_number.is_some()),
        Token::Uint(U256::from(cell.created_number)),
        Token::Uint(U256::from(cell.consumed_number.unwrap_or(0))),
        Token::Uint(U256::from(capacity)),
        Token::Uint(U256::from(lock_hash_type)),
        Token::Uint(U256::from(type_hash_type)),
        Token::Bytes(lock.code_hash().as_bytes().to_vec()),
        Token::Bytes(type_code_hash),
        Token::Bytes(lock.args().as_bytes().to_vec()),
        Token::Bytes(type_args),
        Token::Bytes(cell.cell_data.to_vec()),
    ])])
}

fn encode_default() -> Vec<u8> {
    encode(&[Token::Tuple(vec![
        Token::Bool(false),
        Token::Bool(false),
        Token::Bool(false),
        Token::Uint(U256::from(0)),
        Token::Uint(U256::from(0)),
        Token::Uint(U256::from(0)),
        Token::Uint(U256::from(0)),
        Token::Uint(U256::from(0)),
        Token::Bytes(Vec::new()),
        Token::Bytes(Vec::new()),
        Token::Bytes(Vec::new()),
        Token::Bytes(Vec::new()),
        Token::Bytes(Vec::new()),
    ])])
}
