use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::{err, system_contract::metadata::MetadataHandle};

const INPUT_LEN: usize = 1 + 8;

/// The input argument must includes 9 bytes schema:
/// input[0]: an u8 on behalf of the call type. 0u8 means get metadata by block
/// number and 1u8 means get metadata by epoch number.
/// input[1..9]: 8 bytes of a **big endian** u64 value. If call type is 0u8, it
/// means a block number. If call type is 1u8, it means a epoch number.

#[derive(Default, Clone)]
pub struct Metadata;

impl PrecompileContract for Metadata {
    const ADDRESS: H160 = axon_precompile_address(0x00);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        if let Some(gas) = gas_limit {
            if let Some(limit) = gas_limit {
                if limit < gas {
                    return err!();
                }
            }

            let (ty, number) = parse_input(input)?;

            let metadata = match ty {
                0u8 => MetadataHandle::default()
                    .get_metadata_by_block_number(number)
                    .map_err(|e| err!(_, e.to_string()))?,
                1u8 => MetadataHandle::default()
                    .get_metadata_by_epoch(number)
                    .map_err(|e| err!(_, e.to_string()))?,
                _ => return err!("Invalid call type"),
            };

            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      rlp::encode(&metadata).to_vec(),
                },
                Self::MIN_GAS,
            ));
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}

fn parse_input(input: &[u8]) -> Result<(u8, u64), PrecompileFailure> {
    if input.len() != INPUT_LEN {
        return err!("Invalid input length");
    }

    let mut buf = [0u8; 8];
    buf.copy_from_slice(&input[1..]);

    Ok((input[0], u64::from_le_bytes(buf)))
}
