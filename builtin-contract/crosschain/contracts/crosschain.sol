// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import {IMirrorToken} from "./MirrorToken.sol";
import {IMetadata} from "./Metadata.sol";
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
    CKBToAxonRecord[] private _limitTxes;

    mapping(bytes32 => uint256) _limitTxesMap;
    mapping(address => TokenConfig) private _tokenConfigs;
    mapping(address => bool) private _mirrorTokens;

    event CrossFromCKB(address to, address token, uint256 amount);
    event CrossFromCKBAlert(address to, address token, uint256 amount);
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

    struct CKBToAxonRecord {
        address to;
        address tokenAddress;
        uint256 amount;
        uint256 CKBAmount;
        bytes32 txHash;
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

    function getAndClearlimitTxes()
        external
        view
        returns (CKBToAxonRecord[] memory)
    {
        return _limitTxes;
    }

    function fee(address token, uint256 value) public view returns (uint256) {
        TokenConfig memory config = _tokenConfigs[token];

        return config.feeRatio;
    }

    function _amountReachThreshold(address token, uint256 amount)
        private
        view
        returns (bool)
    {
        return _tokenConfigs[token].threshold <= amount;
    }

    function _deleteLimitTxes(CKBToAxonRecord memory record) private {
        if (_limitTxesMap[record.txHash] == 0) {
            return;
        }

        delete _limitTxes[_limitTxesMap[record.txHash] - 1];
        delete _limitTxesMap[record.txHash];
    }

    function _addLimitTxes(CKBToAxonRecord memory record) private {
        if (_limitTxesMap[record.txHash] > 0) {
            return;
        }

        _limitTxes.push(record);
        _limitTxesMap[record.txHash] = _limitTxes.length;
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

    function _verifySignature() private returns (bool) {
        //TODO
    }

    function _crossATFromCKB(CKBToAxonRecord memory record) private {
        if (record.amount == 0) return;

        payable(record.to).transfer(record.amount);
        emit CrossFromCKB(record.to, AT_ADDRESS, record.amount);
    }

    function _crossCKBFromCKB(CKBToAxonRecord memory record) private {
        if (record.CKBAmount == 0) return;

        IMirrorToken(_wCKB).mint(record.to, record.CKBAmount);

        emit CrossFromCKB(record.to, _wCKB, record.CKBAmount);
    }

    function _crossSUdtFromCKB(CKBToAxonRecord memory record) private {
        if (record.amount == 0) return;

        if (isMirrorToken(record.tokenAddress)) {
            IMirrorToken(record.tokenAddress).mint(record.to, record.amount);
        } else {
            IERC20(record.tokenAddress).transfer(record.to, record.amount);
        }

        emit CrossFromCKB(record.to, record.tokenAddress, record.amount);
    }

    function crossFromCKB(CKBToAxonRecord[] memory records) external {
        require(
            hasRole(RELAYER_ROLE, _msgSender()),
            "CrossChain: must have relayer role"
        );

        require(_verifySignature(), "CrossChain: verify signatures failed");

        require(
            IMetadata(_metadata).isProposer(_msgSender()),
            "CrossChain: replayer must be proposer"
        );

        uint256 length = records.length;
        for (uint256 i = 0; i < length; ++i) {
            CKBToAxonRecord memory record = records[i];
            if (record.amount == 0 && record.CKBAmount == 0) continue;

            if (_amountReachThreshold(_wCKB, record.CKBAmount)) {
                _addLimitTxes(record);

                emit CrossFromCKBAlert(record.to, _wCKB, record.CKBAmount);

                continue;
            } else if (
                _amountReachThreshold(record.tokenAddress, record.amount)
            ) {
                _addLimitTxes(record);

                emit CrossFromCKBAlert(
                    record.to,
                    record.tokenAddress,
                    record.amount
                );

                continue;
            }

            _crossCKBFromCKB(record);

            if (record.tokenAddress == AT_ADDRESS) {
                _crossATFromCKB(record);
            } else {
                _crossSUdtFromCKB(record);
            }
        }
    }
}
