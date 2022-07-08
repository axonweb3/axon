// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

interface IMirrorToken is IERC20 {
    function burn(address from, uint256 amount) external;

    function mint(address to, uint256 amount) external;
}

contract MirrorToken is ERC20, AccessControl, Ownable, IMirrorToken {
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");

    uint8 private _decimals;

    constructor(
        string memory name,
        string memory symbol,
        uint8 decimals_
    ) ERC20(name, symbol) {
        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());
        _decimals = decimals_;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount)
        external
        override
        onlyRole(MANAGER_ROLE)
    {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount)
        external
        override
        onlyRole(MANAGER_ROLE)
    {
        _burn(from, amount);
    }
}
