use evm::ExitSucceed;
use protocol::types::{
    Apply, ApplyBackend, Backend, Basic, ExitReason, ExitRevert, TxResp, H160, H256, U256,
};

use crate::system_contract::{
    CkbLightClientContract, ImageCellContract, MetadataContract, SystemContract,
    CURRENT_HEADER_CELL_ROOT, CURRENT_METADATA_ROOT, HEADER_CELL_ROOT_KEY, METADATA_ROOT_KEY,
};

pub fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     (gas_limit - 1).as_u64(),
        remain_gas:   1u64,
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

pub fn succeed_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Succeed(ExitSucceed::Stopped),
        ret:          vec![],
        gas_used:     0u64,
        remain_gas:   gas_limit.as_u64(),
        fee_cost:     U256::zero(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

pub fn update_mpt_root<B: Backend + ApplyBackend>(backend: &mut B, address: H160) {
    let mut account = backend.basic(address);
    let mut new_storage: Vec<(H256, H256)> = vec![];

    if address == CkbLightClientContract::ADDRESS || address == ImageCellContract::ADDRESS {
        new_storage.push((*HEADER_CELL_ROOT_KEY, **CURRENT_HEADER_CELL_ROOT.load()));
    } else if address == MetadataContract::ADDRESS {
        new_storage.push((*METADATA_ROOT_KEY, **CURRENT_METADATA_ROOT.load()));
    } else {
        unreachable!();
    }
    backend.apply(
        vec![Apply::Modify {
            address,
            basic: Basic {
                balance: account.balance,
                nonce:   account.nonce + U256::one(),
            },
            code: None,
            storage: new_storage.clone(),
            reset_storage: false,
        }],
        vec![],
        false,
    );

    // We need to update the roots of CkbLightClient and ImageCell together
    // so they always equal to each other.
    // But the nounce is only updated for the contract that is called.
    // So we need to keep the nounce of the other contract.
    let other_address: H160;
    if address == CkbLightClientContract::ADDRESS {
        account = backend.basic(ImageCellContract::ADDRESS);
        other_address = ImageCellContract::ADDRESS;
    } else if address == ImageCellContract::ADDRESS {
        account = backend.basic(CkbLightClientContract::ADDRESS);
        other_address = CkbLightClientContract::ADDRESS;
    } else if address == MetadataContract::ADDRESS {
        return;
    } else {
        unreachable!();
    }
    backend.apply(
        vec![Apply::Modify {
            address:       other_address,
            basic:         Basic {
                balance: account.balance,
                nonce:   account.nonce,
            },
            code:          None,
            storage:       new_storage,
            reset_storage: false,
        }],
        vec![],
        false,
    );
}
