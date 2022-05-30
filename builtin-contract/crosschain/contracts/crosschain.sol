// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import {IMirrorToken} from "./MirrorToken.sol";
import {IMetadata} from "./Metadata.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/draft-EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";

contract CrossChain is AccessControl, EIP712 {
    using EnumerableSet for EnumerableSet.AddressSet;

    address public constant AT_ADDRESS = address(0);

    bytes32 public immutable CROSS_FROM_CKB_TYPEHASH;

    uint256 private _epoch = 2**64 - 1;
    uint256 private _relayerThreshold;
    address private _metadata;
    address private _wCKB;
    uint256 private _minWCKB;
    uint256 private _crossFromCKBNonce;
    address[] private _relayers;
    CKBToAxonRecord[] private _limitTxes;

    mapping(address => uint256) _relayersMap;
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
        uint256 threshold,
        address metadata,
        string memory name,
        string memory version
    ) EIP712(name, version) {
        _relayerThreshold = threshold;
        _metadata = metadata;

        CROSS_FROM_CKB_TYPEHASH = keccak256(
            "Transaction(bytes32 recordsHash,uint256 nonce)"
        );
    }

    modifier onlyProposer() {
        require(
            IMetadata(_metadata).isProposer(_msgSender()),
            "CrossChain: sender must be proposer"
        );

        _;
    }

    function _amountReachThreshold(address token, uint256 amount)
        private
        view
        returns (bool)
    {
        return _tokenConfigs[token].threshold <= amount;
    }

    function _removeLimitTxes(CKBToAxonRecord memory record) private {
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

    function _verifyCrossFromCKBSignatures(
        CKBToAxonRecord[] calldata records,
        bytes calldata signatures,
        uint256 nonce
    ) private {
        bytes32 msgHash = _hashTypedDataV4(
            keccak256(
                abi.encode(
                    CROSS_FROM_CKB_TYPEHASH,
                    keccak256(abi.encode(records)),
                    nonce
                )
            )
        );

        _verifySignature(msgHash, signatures);
    }

    function _updateRelayers(address[] memory relayers, uint256 epoch) private {
        if (epoch == _epoch) {
            return;
        }

        uint256 length = relayers.length;
        _relayers = new address[](length);
        for (uint256 i = 0; i < length; ++i) {
            _relayers[i] = relayers[i];
            _relayersMap[relayers[i]] = i + 1;
        }

        _epoch = epoch;
    }

    // will update verifiers and epoch first
    function _verifySignature(bytes32 hash, bytes calldata signatures) private {
        uint256 relayerNumber = signatures.length / 65;

        require(
            relayerNumber >= _relayerThreshold,
            "CrossChain: signatures are not enough"
        );

        (address[] memory relayers, uint256 epoch) = IMetadata(_metadata)
            .verifierList();
        _updateRelayers(relayers, epoch);

        bool[] memory verified = new bool[](relayers.length);
        uint256 threshold = 0;

        for (uint256 i = 0; i < relayerNumber; ++i) {
            (address relayerAddress, ECDSA.RecoverError err) = ECDSA.tryRecover(
                hash,
                signatures[i * 65:(i + 1) * 65]
            );

            if (err == ECDSA.RecoverError.NoError) {
                uint256 relayerIndex = _relayersMap[relayerAddress];

                if (
                    relayerIndex > 0 &&
                    _relayers[relayerIndex - 1] == relayerAddress &&
                    !verified[relayerIndex - 1]
                ) {
                    verified[relayerIndex - 1] = true;
                    if (++threshold > _relayerThreshold) {
                        return;
                    }
                }
            }
        }

        require(
            threshold >= _relayerThreshold,
            "CrossChain: valid signatures are not enough"
        );
    }

    function crossFromCKBNonce() public view returns (uint256) {
        return _crossFromCKBNonce;
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

    function setTokenConfig(address token, TokenConfig calldata config)
        external
        onlyProposer
    {
        _tokenConfigs[token] = config;
    }

    function setWCKB(address token) external onlyProposer {
        _wCKB = token;
    }

    function setWCKBMin(uint256 amount) external onlyProposer {
        _minWCKB = amount;
    }

    function addMirrorToken(address token) public onlyProposer {
        _mirrorTokens[token] = true;
    }

    function isMirrorToken(address token) public view returns (bool) {
        return _mirrorTokens[token];
    }

    // lock AT on Axon network
    function lockAT(bytes32 lockscript) external payable {
        require(msg.value > 0, "CrossChain: value must be more than 0");

        IERC20(_wCKB).transferFrom(_msgSender(), address(this), _minWCKB);

        emit CrossToCKB(lockscript, address(0), msg.value, _minWCKB);
    }

    // tokens are included as follows:
    // lock simple tokens (ERC20) on Axon network
    // burn mirror tokens (sUDTs from CKB network) on Axon network
    // burn wCKB on Axon network
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

    // all the tokens are included as follows:
    // unlock simple tokens (ERC20) on Axon network
    // mint mirror tokens (sUDTs from CKB network) on Axon network
    // mint wCKB (CKB on CKB network)
    // unlock AT on Axon network
    // only proposer can call this method
    // resubmit the tx by using nonce auto increment
    function crossFromCKB(
        CKBToAxonRecord[] calldata records,
        bytes calldata signatures,
        uint256 nonce
    ) external onlyProposer {
        require(_crossFromCKBNonce == nonce, "CrossChain: invalid nonce");

        _verifyCrossFromCKBSignatures(records, signatures, nonce);

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

            _removeLimitTxes(record);
        }

        _crossFromCKBNonce = SafeMath.add(_crossFromCKBNonce, 1);
    }
}
