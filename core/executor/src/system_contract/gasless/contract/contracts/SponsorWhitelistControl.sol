// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

contract SponsorWhitelistControl {

    struct SponsorInfo {
        /// This is the address of the sponsor for gas cost of the contract.
        address sponsor_for_gas;
        /// This is the upper bound of sponsor gas cost per tx.
        uint256 sponsor_gas_bound;
        /// This is the amount of tokens sponsor for gas cost to the contract.
        uint256 sponsor_balance_for_gas;
    }

    struct GaslessInfo {
        SponsorInfo sponsor_info;
        /// This is the accounts in the whitelist which can be sponsored
        mapping(address => bool) whitelist;
    }

    mapping(address => GaslessInfo) gasless_infos;

    /*** Query Functions ***/
    /**
     * @dev get gas sponsor address of specific contract
     * @param contractAddr The address of the sponsored contract
     */
    function getSponsorForGas(address contractAddr) public view returns (address) {
        return gasless_infos[contractAddr].sponsor_info.sponsor_for_gas;
    }

    /**
     * @dev get current Sponsored Balance for gas
     * @param contractAddr The address of the sponsored contract
     */
    function getSponsoredBalanceForGas(address contractAddr) public view returns (uint256) {
        return gasless_infos[contractAddr].sponsor_info.sponsor_balance_for_gas;
    }

    /**
     * @dev get current Sponsored Gas fee upper bound
     * @param contractAddr The address of the sponsored contract
     */
    function getSponsoredGasFeeUpperBound(address contractAddr) public view returns (uint256) {
        return gasless_infos[contractAddr].sponsor_info.sponsor_gas_bound;
    }

    /**
     * @dev check if a user is in a contract's whitelist
     * @param contractAddr The address of the sponsored contract
     * @param user The address of contract user
     */
    function isWhitelisted(address contractAddr, address user) external view returns (bool) {
        return gasless_infos[contractAddr].whitelist[user];
    }

    /*
     * @dev check if all users are in a contract's whitelist
     * @param contractAddr The address of the sponsored contract
     */
    // function isAllWhitelisted(address contractAddr) public view returns (bool) {}

    function update_sponsor_info(SponsorInfo storage old_sponsor_info, SponsorInfo memory new_sponsor_info) private {
        if (old_sponsor_info.sponsor_for_gas == new_sponsor_info.sponsor_for_gas) {
            old_sponsor_info.sponsor_balance_for_gas += new_sponsor_info.sponsor_balance_for_gas;
            // increase gas bound if larger
            if (old_sponsor_info.sponsor_gas_bound < new_sponsor_info.sponsor_gas_bound) {
                old_sponsor_info.sponsor_gas_bound = new_sponsor_info.sponsor_gas_bound;
            }
        } else {
            require(new_sponsor_info.sponsor_balance_for_gas >= old_sponsor_info.sponsor_balance_for_gas, "New Sponsor balance less than old");
            // if current sponsor is not capacable of paying at least one tx, then update sponsor info whatever
            // otherwise, the 
            if (old_sponsor_info.sponsor_balance_for_gas >= old_sponsor_info.sponsor_gas_bound) {
                require(new_sponsor_info.sponsor_gas_bound >= old_sponsor_info.sponsor_gas_bound, "New Sponsor gas bound less than old");
            }
            // the new sponsored balance should be accumulated or returned to the original sponsor?
            old_sponsor_info.sponsor_balance_for_gas += new_sponsor_info.sponsor_balance_for_gas;
            old_sponsor_info.sponsor_for_gas = new_sponsor_info.sponsor_for_gas;
            old_sponsor_info.sponsor_gas_bound = new_sponsor_info.sponsor_gas_bound;
        }
    }

    // ------------------------------------------------------------------------
    // Someone will sponsor the gas cost for contract `contractAddr` with an
    // `upper_bound` for a single transaction.
    // ------------------------------------------------------------------------
    function setSponsorForGas(address contractAddr, uint upperBound) public payable {
        uint256 sponsor_balance_for_gas = msg.value;

        require(upperBound >= 1000, "Sponsor upper bound less than minimum");
        require(sponsor_balance_for_gas >= upperBound, "Sponsor balance less than upper bound");

        // The contractAddr has been sponsored before
        if(gasless_infos[contractAddr].sponsor_info.sponsor_gas_bound > 0) {
            SponsorInfo memory new_sponsor_info;
            new_sponsor_info.sponsor_balance_for_gas = sponsor_balance_for_gas;
            new_sponsor_info.sponsor_for_gas = msg.sender;
            new_sponsor_info.sponsor_gas_bound = upperBound;
            update_sponsor_info(gasless_infos[contractAddr].sponsor_info, new_sponsor_info);
        } else {
            gasless_infos[contractAddr].sponsor_info.sponsor_balance_for_gas = sponsor_balance_for_gas;
            gasless_infos[contractAddr].sponsor_info.sponsor_for_gas = msg.sender;
            gasless_infos[contractAddr].sponsor_info.sponsor_gas_bound = upperBound;
        }
    }

    function substractSponsorBalance(address contractAddr, uint balance) public payable {
        require(gasless_infos[contractAddr].sponsor_info.sponsor_balance_for_gas >= balance, "Not enough sponsored balance");
        gasless_infos[contractAddr].sponsor_info.sponsor_balance_for_gas -= balance;
    }

    // ------------------------------------------------------------------------
    // Add commission privilege for address `user` to some contract.
    // ------------------------------------------------------------------------
    function addPrivilege(address[] memory users) public {
        // The sponsored contract will call the method
        address contractAddr = msg.sender;
        for(uint i = 0; i < users.length; ++i) {
            gasless_infos[contractAddr].whitelist[users[i]] = true;
        }
    }

    // ------------------------------------------------------------------------
    // Remove commission privilege for address `user` from some contract.
    // ------------------------------------------------------------------------
    function removePrivilege(address[] memory users) public {
        // The sponsored contract will call the method
        address contractAddr = msg.sender;
        for(uint i = 0; i < users.length; ++i) {
            delete gasless_infos[contractAddr].whitelist[users[i]];
        }
    }
}