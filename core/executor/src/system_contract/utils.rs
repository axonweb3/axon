use evm::ExitSucceed;

use protocol::types::{
    Apply, ApplyBackend, Backend, ExitReason, ExitRevert, TxResp, H160, H256, U256,
};

use crate::system_contract::{
    CKB_LIGHT_CLIENT_CONTRACT_ADDRESS, HEADER_CELL_ROOT_KEY, IMAGE_CELL_CONTRACT_ADDRESS,
    METADATA_CONTRACT_ADDRESS, METADATA_ROOT_KEY,
};
use crate::{CURRENT_HEADER_CELL_ROOT, CURRENT_METADATA_ROOT};

pub fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     (gas_limit - 1).low_u64(),
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
        remain_gas:   gas_limit.low_u64(),
        fee_cost:     U256::zero(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

pub fn update_states<B: Backend + ApplyBackend>(
    backend: &mut B,
    sender: H160,
    contract_address: H160,
) {
    let mut changes = generate_mpt_root_changes(backend, contract_address);
    changes.append(&mut generate_sender_changes(backend, sender));
    backend.apply(changes, vec![], false);
}

pub fn generate_sender_changes<B: Backend + ApplyBackend>(
    backend: &mut B,
    sender: H160,
) -> Vec<Apply<Vec<(H256, H256)>>> {
    let mut account = backend.basic(sender);
    account.nonce += U256::one();
    vec![Apply::Modify {
        address:       sender,
        basic:         account,
        code:          None,
        storage:       vec![],
        reset_storage: false,
    }]
}

pub fn generate_mpt_root_changes<B: Backend + ApplyBackend>(
    backend: &mut B,
    contract_address: H160,
) -> Vec<Apply<Vec<(H256, H256)>>> {
    if contract_address == CKB_LIGHT_CLIENT_CONTRACT_ADDRESS
        || contract_address == IMAGE_CELL_CONTRACT_ADDRESS
    {
        let current_header_cell_root = CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow());
        let storage_changes = vec![(*HEADER_CELL_ROOT_KEY, current_header_cell_root)];
        vec![
            Apply::Modify {
                address:       CKB_LIGHT_CLIENT_CONTRACT_ADDRESS,
                basic:         backend.basic(CKB_LIGHT_CLIENT_CONTRACT_ADDRESS),
                code:          None,
                storage:       storage_changes.clone(),
                reset_storage: false,
            },
            Apply::Modify {
                address:       IMAGE_CELL_CONTRACT_ADDRESS,
                basic:         backend.basic(IMAGE_CELL_CONTRACT_ADDRESS),
                code:          None,
                storage:       storage_changes,
                reset_storage: false,
            },
        ]
    } else if contract_address == METADATA_CONTRACT_ADDRESS {
        let current_metadata_root = CURRENT_METADATA_ROOT.with(|r| *r.borrow());
        vec![Apply::Modify {
            address:       METADATA_CONTRACT_ADDRESS,
            basic:         backend.basic(METADATA_CONTRACT_ADDRESS),
            code:          None,
            storage:       vec![(*METADATA_ROOT_KEY, current_metadata_root)],
            reset_storage: false,
        }]
    } else {
        unreachable!()
    }
}
