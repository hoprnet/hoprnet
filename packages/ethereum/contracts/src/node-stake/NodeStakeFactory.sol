// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import "openzeppelin-contracts-upgradeable/proxy/ClonesUpgradeable.sol";
import "openzeppelin-contracts/utils/Address.sol";
import "../../script/utils/SafeSuiteLib.sol";
import "safe-contracts/proxies/SafeProxy.sol";
import "safe-contracts/proxies/SafeProxyFactory.sol";
import "safe-contracts/Safe.sol";
import "safe-contracts/common/Enum.sol";

contract HoprNodeStakeFactory {
    using Address for address;
    using ClonesUpgradeable for address;

    address internal constant SENTINEL_OWNERS = address(0x1);
    bytes32 internal r;
    bytes internal approvalHashSig;

    error TooFewOwners();

    event NewHoprNodeStakeModule(address instance);
    event NewHoprNodeStakeSafe(address instance);

    constructor() {
        r = bytes32(uint256(uint160(address(this))));
        approvalHashSig = abi.encodePacked(abi.encode(r, bytes32(0)), bytes1(hex"01"));
    }

    /**
     * @dev Returns the version of Safe deployments
     */
    function safeVersion() public pure returns (string memory) {
        return SafeSuiteLib.SAFE_VERSION;
    }

    /**
     * @dev Create a safe proxy and a module proxy
     * @param moduleSingletonAddress singleton contract of Safe
     * @param admins owners of safe by default it's 1 out of n
     * @param nonce nonce to create salt
     * @param defaultTarget default target (see TargetUtils.sol) for the current HoprChannels (and HoprToken) contract
     */
    function clone(address moduleSingletonAddress, address[] memory admins, uint256 nonce, bytes32 defaultTarget)
        public
        returns (address, address payable)
    {
        // check on provided admin array
        if (admins.length == 0) {
            revert TooFewOwners();
        }
        // set the salt
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        // 1. Deploy node management module
        address moduleProxy = moduleSingletonAddress.cloneDeterministic(salt);

        // swap one owner for factory
        address admin0 = admins[0];
        admins[0] = address(this);

        // prepare safe initializer;
        bytes memory safeInitializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            SafeSuiteLib.SAFE_CompatibilityFallbackHandler_ADDRESS,
            address(0),
            0,
            address(0)
        );

        // 2. deploy safe proxy
        SafeProxy safeProxy = SafeProxyFactory(SafeSuiteLib.SAFE_SafeProxyFactory_ADDRESS).createProxyWithNonce(
            SafeSuiteLib.SAFE_Safe_ADDRESS, safeInitializer, nonce
        );
        address payable safeProxyAddr = payable(address(safeProxy));

        // add Safe and multisend to the module, and transfer the ownership to module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(address(safeProxy), SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS, defaultTarget)
        );
        moduleProxy.functionCall(moduleInitializer);

        // enable node management module
        bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
        prepareSafeTx(Safe(safeProxyAddr), 0, enableModuleData);
        // swap owner for Safe
        bytes memory swapOwnerData =
            abi.encodeWithSignature("swapOwner(address,address,address)", SENTINEL_OWNERS, address(this), admin0);
        prepareSafeTx(Safe(safeProxyAddr), 1, swapOwnerData);

        emit NewHoprNodeStakeModule(moduleProxy);
        emit NewHoprNodeStakeSafe(address(safeProxy));
        safeProxyAddr = payable(address(safeProxy));
        return (moduleProxy, safeProxyAddr);
    }

    function prepareSafeTx(Safe safe, uint256 nonce, bytes memory data) private {
        bytes32 dataHash =
            safe.getTransactionHash(address(safe), 0, data, Enum.Operation.Call, 0, 0, 0, address(0), msg.sender, nonce);
        safe.approveHash(dataHash);
        safe.execTransaction(
            address(safe),
            0,
            data,
            Enum.Operation.Call,
            0,
            0,
            0,
            address(0),
            payable(address(msg.sender)),
            approvalHashSig
        );
    }

    function predictDeterministicAddress(address implementation, bytes32 salt)
        public
        view
        returns (address predicted)
    {
        return implementation.predictDeterministicAddress(salt);
    }
}
