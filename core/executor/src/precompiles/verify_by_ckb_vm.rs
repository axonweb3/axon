use ethers::contract::{EthAbiCodec, EthAbiType};
use ethers::{abi::AbiDecode, core::types::Bytes as EthBytes};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::traits::Interoperation;
use protocol::types::{SignatureR, SignatureS, H160, H256};

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};

use crate::precompiles::{axon_precompile_address, call_ckb_vm::CellDep, PrecompileContract};
use crate::system_contract::{image_cell::image_cell_abi::OutPoint, DataProvider};
use crate::{err, CURRENT_HEADER_CELL_ROOT};

#[derive(Default, Clone)]
pub struct CkbVM;

impl PrecompileContract for CkbVM {
    const ADDRESS: H160 = axon_precompile_address(0x05);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        let payload = parse_input(input)?;

        if let Some(gas) = gas_limit {
            let res = InteroperationImpl::verify_by_ckb_vm(
                Default::default(),
                DataProvider::new(CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow())),
                &InteroperationImpl::dummy_transaction(
                    SignatureR::new_by_ref(
                        payload.cell_deps(),
                        payload.header_deps(),
                        payload.inputs(),
                        Default::default(),
                    ),
                    SignatureS::new(payload.witnesses()),
                    None,
                ),
                None,
                gas_to_cycle(gas),
            )
            .map_err(|e| err!(_, e.to_string()))?;

            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      res.to_le_bytes().to_vec(),
                },
                cycle_to_gas(res).max(Self::MIN_GAS),
            ));
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        unreachable!()
    }
}

fn parse_input(input: &[u8]) -> Result<VerifyByCkbPayload, PrecompileFailure> {
    <VerifyByCkbPayload as AbiDecode>::decode(input).map_err(|_| err!(_, "decode input"))
}

#[derive(EthAbiType, EthAbiCodec, Clone, Default, Debug, PartialEq, Eq)]
pub struct VerifyByCkbPayload {
    pub cell_deps:   Vec<CellDep>,
    pub header_deps: Vec<[u8; 32]>,
    pub inputs:      Vec<OutPoint>,
    pub witnesses:   Vec<WitnessArgs>,
}

impl VerifyByCkbPayload {
    pub fn cell_deps(&self) -> Vec<protocol::types::CellDep> {
        self.cell_deps
            .iter()
            .map(|c| protocol::types::CellDep {
                tx_hash:  c.out_point.tx_hash.into(),
                index:    c.out_point.index,
                dep_type: c.dep_type,
            })
            .collect()
    }

    pub fn header_deps(&self) -> Vec<H256> {
        self.header_deps.iter().map(Into::into).collect()
    }

    pub fn inputs(&self) -> Vec<protocol::types::OutPoint> {
        self.inputs
            .iter()
            .map(|i| protocol::types::OutPoint {
                tx_hash: i.tx_hash.into(),
                index:   i.index,
            })
            .collect()
    }

    pub fn witnesses(&self) -> Vec<protocol::types::Witness> {
        self.witnesses.clone().into_iter().map(Into::into).collect()
    }
}

#[derive(EthAbiType, EthAbiCodec, Clone, Default, Debug, PartialEq, Eq)]
pub struct WitnessArgs {
    pub lock:        EthBytes,
    pub input_type:  EthBytes,
    pub output_type: EthBytes,
}

impl From<WitnessArgs> for protocol::types::Witness {
    fn from(w: WitnessArgs) -> Self {
        let lock = if w.lock.is_empty() {
            None
        } else {
            Some(w.lock.0)
        };
        let input_type = if w.input_type.is_empty() {
            None
        } else {
            Some(w.input_type.0)
        };
        let output_type = if w.output_type.is_empty() {
            None
        } else {
            Some(w.output_type.0)
        };

        protocol::types::Witness {
            lock,
            input_type,
            output_type,
        }
    }
}
