// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

// import "hardhat/console.sol";
import {IMirrorToken} from "./MirrorToken.sol";
import "@openzeppelin/contracts/access/AccessControlEnumerable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/draft-EIP712.sol";

contract CrossChain is AccessControlEnumerable, EIP712 {
    bytes32 public constant RELAYER_ROLE = keccak256("RELAYER_ROLE");

    address public constant AT_ADDRESS = address(0);
    uint256 private _relayerThreshold;
    address private _metadata;
    address private _wCKB;
    uint256 private _minWCKB;

    mapping(address => TokenConfig) private _tokenConfigs;
    mapping(address => bool) private _mirrorTokens;

    event CrossToCKB(
        bytes32 lockscript,
        address token,
        uint256 amount,
        uint256 minWCKBAmount
    );

    struct TokenConfig {
        uint256 feeRatio;
        uint256 threshold;
    }

    struct AxonToCKBREcord {
        bytes32 lockscriptHash;
        address tokenAddress;
        uint256 amount;
    }

    constructor(
        address[] memory relayers,
        uint256 threshold,
        address metadata,
        string memory name,
        string memory version
    ) EIP712(name, version) {
        uint256 length = relayers.length;
        for (uint256 i = 0; i < length; ++i) {
            _setupRole(RELAYER_ROLE, relayers[i]);
        }
        _setRoleAdmin(RELAYER_ROLE, RELAYER_ROLE);

        _relayerThreshold = threshold;
        _metadata = metadata;
    }

    function setTokenConfig(address token, TokenConfig calldata config)
        external
    {
        require(
            hasRole(RELAYER_ROLE, _msgSender()),
            "CrossChain: must have relayer role"
        );

        _tokenConfigs[token] = config;
    }

    function setWCKB(address token) external {
        require(
            hasRole(RELAYER_ROLE, _msgSender()),
            "CrossChain: must have relayer role"
        );

        _wCKB = token;
    }

    function setWCKBMin(uint256 amount) external {
        require(
            hasRole(RELAYER_ROLE, _msgSender()),
            "CrossChain: must have relayer role"
        );

        _minWCKB = amount;
    }

    function addMirrorToken(address token) public {
        require(
            hasRole(RELAYER_ROLE, _msgSender()),
            "CrossChain: must have relayer role"
        );

        _mirrorTokens[token] = true;
    }

    function isMirrorToken(address token) public view returns (bool) {
        return _mirrorTokens[token];
    }

    function lockAT(bytes32 lockscript) external payable {
        require(msg.value > 0, "CrossChain: value must be more than 0");

        IERC20(_wCKB).transferFrom(_msgSender(), address(this), _minWCKB);

        emit CrossToCKB(lockscript, address(0), msg.value, _minWCKB);
    }

    function crossTokenToCKB(
        bytes32 lockscript,
        address token,
        uint256 amount
    ) external {
        require(amount > 0, "CrossChain: amount must be more than 0");

        require(
            IERC20(_wCKB).balanceOf(_msgSender()) > _minWCKB,
            "CrossChain: amount of wckb is insufficient"
        );

        if (isMirrorToken(token)) {
            if (token == _wCKB) {
                amount -= _minWCKB;
            }

            IMirrorToken(token).burn(_msgSender(), amount);
        } else {
            IERC20(token).transferFrom(_msgSender(), address(this), amount);
        }

        IERC20(_wCKB).transferFrom(_msgSender(), address(this), _minWCKB);

        emit CrossToCKB(lockscript, token, amount, _minWCKB);
    }
}
