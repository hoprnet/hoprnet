// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.7.0 <0.9.0;

import "@openzeppelin/contracts-upgradeable/proxy/ClonesUpgradeable.sol";
import "openzeppelin-contracts-4.8.3/utils/Address.sol";
import "openzeppelin-contracts-4.8.3/access/Ownable.sol";
import "../../script/utils/SafeSuiteLib.sol";
import "safe-contracts/proxies/SafeProxy.sol";
import "safe-contracts/proxies/SafeProxyFactory.sol";

contract HoprNodeStakeFactory is Ownable {
    using Address for address;
    using ClonesUpgradeable for address;

    event NewHoprNodeStakeModule(address instance);
    event NewHoprNodeStakeSafe(address instance);

    function clone(address moduleSingletonAddress, address adminKey, uint256 nonce) public {
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        address moduleProxy = moduleSingletonAddress.cloneDeterministic(salt);

        // prepare safe initializer; temporarily add this as an owner
        bytes memory safeInitializer = abi.encodeWithSignature(
            'setup(address[],uint256,address,bytes,address,address,uint256,address)', 
            [adminKey, address(this)], 
            1, 
            address(0), 
            hex"00", 
            SafeSuiteLib.SAFE_TokenCallbackHandler_ADDRESS, 
            address(0), 
            0,
            address(0)
        );
        SafeProxy safeProxy = SafeProxyFactory(SafeSuiteLib.SAFE_SafeProxyFactory_ADDRESS).createProxyWithNonce(SafeSuiteLib.SAFE_Safe_ADDRESS, safeInitializer, nonce);

        // add Safe and multisend to the module, and transfer the ownership to module
        bytes memory moduleInitializer = abi.encodeWithSignature("initialize(bytes)", abi.encode(address(safeProxy), SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS));
        moduleProxy.functionCall(moduleInitializer);
    }

    // function clone(address implementation, bytes calldata initdata) public {
    //     _initAndEmit(implementation.clone(), initdata);
    // }

    function cloneDeterministic(
        address implementation,
        bytes32 salt,
        bytes calldata initdata
    ) public payable {
        _initAndEmit(implementation.cloneDeterministic(salt), initdata);
    }

    function predictDeterministicAddress(address implementation, bytes32 salt) public view returns (address predicted) {
        return implementation.predictDeterministicAddress(salt);
    }

    function _initAndEmit(address instance, bytes memory initdata) private {
        if (initdata.length > 0) {
            instance.functionCallWithValue(initdata, msg.value);
        }
        emit NewHoprNodeStakeModule(instance);
    }
}

