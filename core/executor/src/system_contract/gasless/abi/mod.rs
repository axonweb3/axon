use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use protocol::types::{
    Apply, ApplyBackend, Backend, Basic, ExitReason, ExitRevert, ExitSucceed, SignedTransaction,
    TxResp, H160, H256, U256,
};

use crate::system_contract::{system_contract_address, SystemContract};
use ethers::abi::AbiDecode;
pub mod sponsor_whitelist_control_abi;

pub const ADDRESS: H160 = system_contract_address(0x1);
#[derive(Debug, Default)]
pub struct GaslessInfo {
    /// This is the address of the sponsor for gas cost of the contract.
    pub sponsor_for_gas:         H160,
    /// This is the upper bound of sponsor gas cost per tx.
    pub sponsor_gas_bound:       U256,
    /// This is the amount of tokens sponsor for gas cost to the contract.
    pub sponsor_balance_for_gas: U256,
    /// This is the whitelist which are sponsored to the contract.
    pub whitelist:               HashSet<H160>,
}

#[derive(Debug, Default)]
pub struct GaslessContract {
    // HashMap key is the address of the contract to be sponsored
    pub sponsor_infos: RefCell<HashMap<H160, GaslessInfo>>,
}

impl GaslessContract {
    pub fn new() -> Self {
        GaslessContract {
            sponsor_infos: RefCell::new(HashMap::new()),
        }
    }
}

pub fn set_sponsor_for_gas(
    gasless_infos: &mut HashMap<H160, GaslessInfo>,
    sponsor: H160,
    balance: &U256,
    data: sponsor_whitelist_control_abi::SetSponsorForGasCall,
) -> Result<(), String> {
    if balance < &data.upper_bound {
        return Err(String::from("Balance less than upper_bound"));
    }
    if gasless_infos.contains_key(&data.contract_addr) {
        let gasless_info = gasless_infos.get_mut(&data.contract_addr).unwrap();
        if gasless_info.sponsor_for_gas.is_zero() {
            gasless_info.sponsor_for_gas = sponsor;
            gasless_info.sponsor_balance_for_gas = *balance;
            gasless_info.sponsor_gas_bound = data.upper_bound;
        } else {
            // if the same sponsor, add the balance directly
            if gasless_info.sponsor_for_gas == sponsor {
                gasless_info.sponsor_balance_for_gas += *balance;
                if data.upper_bound > gasless_info.sponsor_gas_bound {
                    gasless_info.sponsor_gas_bound = data.upper_bound;
                }
            } else {
                // The new sponsor's balance must be larger than the current one
                if *balance <= gasless_info.sponsor_balance_for_gas {
                    return Err(String::from("New sponsor balance less than current!"));
                }
                if gasless_info.sponsor_balance_for_gas >= gasless_info.sponsor_gas_bound
                    && data.upper_bound < gasless_info.sponsor_gas_bound
                {
                    return Err(String::from("upper_bound is not exceed previous sponsor!"));
                }

                gasless_info.sponsor_for_gas = sponsor;
                // the new sponsored balance should be accumulated or returned to the original
                // sponsor?
                gasless_info.sponsor_balance_for_gas += *balance;
                gasless_info.sponsor_gas_bound = data.upper_bound;
            }
        }
    } else {
        let gasless_info = GaslessInfo {
            sponsor_for_gas:         sponsor,
            sponsor_gas_bound:       data.upper_bound,
            sponsor_balance_for_gas: *balance,
            whitelist:               HashSet::default(),
        };
        gasless_infos.insert(data.contract_addr, gasless_info);
    }

    Ok(())
}

pub fn add_privilege(
    gasless_infos: &mut HashMap<H160, GaslessInfo>,
    contract_addr: H160,
    data: sponsor_whitelist_control_abi::AddPrivilegeCall,
) -> Result<(), String> {
    if !gasless_infos.contains_key(&contract_addr) {
        gasless_infos.insert(contract_addr, GaslessInfo::default());
    }

    let gasless_info = gasless_infos.get_mut(&contract_addr).unwrap();
    for user in data.users {
        gasless_info.whitelist.insert(user);
    }

    Ok(())
}

pub fn remove_privilege(
    gasless_infos: &mut HashMap<H160, GaslessInfo>,
    contract_addr: H160,
    data: sponsor_whitelist_control_abi::RemovePrivilegeCall,
) -> Result<(), String> {
    let gasless_info = gasless_infos.get_mut(&contract_addr).unwrap();
    for user in data.users {
        gasless_info.whitelist.remove(&user);
    }

    Ok(())
}

impl SystemContract for GaslessContract {
    const ADDRESS: H160 = system_contract_address(0x1);
    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp {
        let sender = tx.sender;
        let tx = &tx.transaction.unsigned;
        let tx_data = tx.data();

        match sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::decode(tx_data) {
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::AddPrivilege(data)) => {
                // todo
                let mut gasless_infos = self.sponsor_infos.borrow_mut();
                // only the contract owner can change the whitelist stored in SponsorWhitelistCtrl
                let _result = add_privilege(&mut gasless_infos, sender, data);
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::GetSponsorForGas(_)) => {
                // todo
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::GetSponsoredBalanceForGas(_)) => {
                // todo
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::GetSponsoredGasFeeUpperBound(_)) => {
                // todo
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::IsAllWhitelisted(_)) => {
                // todo
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::IsWhitelisted(_)) => {
                // todo
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::RemovePrivilege(data)) => {
                // todo
                let mut gasless_infos = self.sponsor_infos.borrow_mut();
                // only the contract owner can change the whitelist stored in SponsorWhitelistCtrl
                let _result = remove_privilege(&mut gasless_infos, sender, data);
            }
            Ok(sponsor_whitelist_control_abi::SponsorWhitelistControlCalls::SetSponsorForGas(data)) => {
                // todo
                let mut sponsor_infos = self.sponsor_infos.borrow_mut();
                let _result = set_sponsor_for_gas(&mut sponsor_infos , sender, tx.value(), data);
            }
            Err(_) => {
                return revert_resp(*tx.gas_limit());
            }
        }

        TxResp {
            exit_reason:  ExitReason::Succeed(ExitSucceed::Returned),
            ret:          vec![],
            gas_used:     0u64,
            remain_gas:   tx.gas_limit().as_u64(),
            fee_cost:     U256::zero(),
            logs:         vec![],
            code_address: None,
            removed:      false,
        }
    }
}

impl GaslessContract {
    fn update_sponsor_infos<B: Backend + ApplyBackend>(
        &self,
        backend: &mut B,
        _gasless_infos: HashMap<H160, GaslessInfo>,
    ) -> Result<(), String> {
        let account = backend.basic(GaslessContract::ADDRESS);

        backend.apply(
            vec![Apply::Modify {
                address:       GaslessContract::ADDRESS,
                basic:         Basic {
                    balance: account.balance,
                    nonce:   account.nonce + U256::one(),
                },
                code:          None,
                storage:       vec![(H256::default(), H256::default())],
                reset_storage: false,
            }],
            vec![],
            false,
        );
        Ok(())
    }
}

fn revert_resp(gas_limit: U256) -> TxResp {
    TxResp {
        exit_reason:  ExitReason::Revert(ExitRevert::Reverted),
        ret:          vec![],
        gas_used:     1u64,
        remain_gas:   (gas_limit - 1).as_u64(),
        fee_cost:     U256::one(),
        logs:         vec![],
        code_address: None,
        removed:      false,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_set_sponsor_for_gas() {
        let mut gasless_infos: HashMap<H160, GaslessInfo> = HashMap::new();
        assert!(gasless_infos.is_empty());
        let contract_addr = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();

        let gas_bound = U256::from(10000);
        let data = sponsor_whitelist_control_abi::SetSponsorForGasCall {
            contract_addr: contract_addr,
            upper_bound:   gas_bound,
        };

        let sponsor = H160::from_str("0x0017f1962b36e491b30a40b2405849e597b00001").unwrap();
        let unenough_balance = U256::from(500);
        let result = set_sponsor_for_gas(&mut gasless_infos, sponsor, &unenough_balance, data.clone());
        assert_eq!(result, Err(String::from("Balance less than upper_bound")));

        let enough_balance = U256::from(150000);
        let result = set_sponsor_for_gas(&mut gasless_infos, sponsor, &enough_balance, data);
        assert_eq!(result, Ok(()));
        assert!(!gasless_infos.is_empty());
        assert_eq!(gasless_infos[&contract_addr].sponsor_for_gas, sponsor);
        assert_eq!(gasless_infos[&contract_addr].sponsor_balance_for_gas, enough_balance);
        assert_eq!(gasless_infos[&contract_addr].sponsor_gas_bound, gas_bound);
    }

    #[test]
    fn test_add_priviledge() {
        let mut gasless_infos: HashMap<H160, GaslessInfo> = HashMap::new();
        assert!(gasless_infos.is_empty());
        let contract_addr = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();

        let user1 = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597b00001").unwrap();
        let data = sponsor_whitelist_control_abi::AddPrivilegeCall { users: vec![user1] };

        let _result = add_privilege(&mut gasless_infos, contract_addr, data);

        assert!(!gasless_infos.is_empty());
        let users = HashSet::from([user1]);
        assert_eq!(gasless_infos[&contract_addr].whitelist, users);
    }

    #[test]
    fn test_remove_priviledge() {
        let sponsor = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5f01").unwrap();
        let contract_addr = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597ba5fb5").unwrap();
        let mut gasless_infos: HashMap<H160, GaslessInfo> = HashMap::new();
        let user1 = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597b00001").unwrap();
        let gasless_info = GaslessInfo {
            sponsor_balance_for_gas: U256::zero(),
            sponsor_for_gas:         sponsor,
            sponsor_gas_bound:       U256::zero(),
            whitelist:               HashSet::from([user1]),
        };
        gasless_infos.insert(contract_addr, gasless_info);

        {
            let user2 = H160::from_str("0x3f17f1962b36e491b30a40b2405849e597b00002").unwrap();
            let data = sponsor_whitelist_control_abi::RemovePrivilegeCall { users: vec![user2] };

            let _result = remove_privilege(&mut gasless_infos, contract_addr, data);
            let users = HashSet::from([user1]);
            assert_eq!(gasless_infos[&contract_addr].whitelist, users);
        }

        {
            let data = sponsor_whitelist_control_abi::RemovePrivilegeCall { users: vec![user1] };

            let _result = remove_privilege(&mut gasless_infos, contract_addr, data);
            assert!(gasless_infos[&contract_addr].whitelist.is_empty());
        }
    }
}
