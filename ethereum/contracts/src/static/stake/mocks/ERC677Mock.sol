// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import "openzeppelin-contracts-4.4.2/token/ERC20/ERC20.sol";
import "openzeppelin-contracts-4.4.2/utils/Address.sol";

/**
 * @title ERC677BridgeToken
 * @dev The basic implementation of a bridgeable ERC677-compatible token
 */
contract ERC677Mock is ERC20 {
    using Address for address;

    bytes4 internal constant ON_TOKEN_TRANSFER = 0xa4c0ed36; // onTokenTransfer(address,uint256,bytes)

    event TransferAndCall(address indexed from, address indexed to, uint256 value, bytes data);

    constructor() ERC20("ERC677Mock", "M677") {}

    modifier validRecipient(address _recipient) {
        require(_recipient != address(0) && _recipient != address(this));
        /* solcov ignore next */
        _;
    }

    function batchMintInternal(address[] memory _to, uint256 _value) external {
        for (uint256 index = 0; index < _to.length; index++) {
            _mint(_to[index], _value);
        }
    }

    function transferAndCall(address _to, uint256 _value, bytes memory _data)
        external
        validRecipient(_to)
        returns (bool)
    {
        require(super.transfer(_to, _value));
        emit TransferAndCall(msg.sender, _to, _value, _data);

        if (_to.isContract()) {
            (bool success,) = _to.call(abi.encodeWithSelector(ON_TOKEN_TRANSFER, msg.sender, _value, _data));
            require(success, "ERC677Mock: failed when calling onTokenTransfer");
        }
        return true;
    }

    function superTransfer(address _to, uint256 _value) internal returns (bool) {
        return super.transfer(_to, _value);
    }

    function transfer(address _to, uint256 _value) public override returns (bool) {
        require(super.transfer(_to, _value));
        (bool success,) = _to.call(abi.encodeWithSelector(ON_TOKEN_TRANSFER, msg.sender, _value, new bytes(0)));
        require(success);
        return true;
    }

    function transferFrom(address _from, address _to, uint256 _value) public override returns (bool) {
        require(super.transferFrom(_from, _to, _value));
        (bool success,) = _to.call(abi.encodeWithSelector(ON_TOKEN_TRANSFER, msg.sender, _value, new bytes(0)));
        require(success);
        return true;
    }
}
