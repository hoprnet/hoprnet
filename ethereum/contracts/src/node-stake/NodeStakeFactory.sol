// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable/proxy/ClonesUpgradeable.sol";
import { Address } from "openzeppelin-contracts/utils/Address.sol";
import { SafeSuiteLib } from "../utils/SafeSuiteLib.sol";
import { SafeProxy } from "safe-contracts/proxies/SafeProxy.sol";
import { SafeProxyFactory } from "safe-contracts/proxies/SafeProxyFactory.sol";
import { Safe } from "safe-contracts/Safe.sol";
import { Enum } from "safe-contracts/common/Enum.sol";

abstract contract HoprNodeStakeFactoryEvents {
    event NewHoprNodeStakeModule(address indexed moduleImplementation, address instance); // Emit when a new module is
        // created
    event NewHoprNodeStakeSafe(address instance); // Emit when a new safe proxy is created
}

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
 * @title HoprNodeStakeFactory
 * @dev This contract is responsible for deploying a 1-of-n Safe proxy and a module proxy for HOPR node management.
 * The factory contract handles the deployment and initialization of these proxies.
 */
contract HoprNodeStakeFactory is HoprNodeStakeFactoryEvents {
    using Address for address;
    using ClonesUpgradeable for address;

    // A sentinel address that serves as the start pointer of the owner linked list used in the OwnerManager of
    // safe-contracts
    address internal constant SENTINEL_OWNERS = address(0x1);

    // Encoded address of the contract's approver, used for EIP-1271 signature verification
    bytes32 internal immutable r;

    // Signature of the approved hash used for EIP-1271 signature verification
    bytes internal approvalHashSig;

    // Error indicating that there are too few owners provided
    error TooFewOwners();

    /**
     * @dev Constructor function to initialize contract state.
     * Initializes the encoded address of the contract's approver and the approved hash signature.
     */
    constructor() {
        // Encode the contract's address to be used in EIP-1271 signature verification
        r = bytes32(uint256(uint160(address(this))));

        // Encode the EIP-1271 contract signature for approval hash verification
        approvalHashSig = abi.encodePacked(abi.encode(r, bytes32(0)), bytes1(hex"01"));
    }

    /**
     * @dev Returns the version of Safe deployments
     */
    function safeVersion() public pure returns (string memory) {
        return SafeSuiteLib.SAFE_VERSION;
    }

    /**
     * @dev Deploys a 1-of-n Safe proxy and a module proxy for HOPR node management.
     * @param moduleSingletonAddress The singleton contract address of the HOPR node management module, as defined in
     * the `HoprNodeManagementModule` contract.
     * @param admins The list of owners for the Safe proxy. The multisig threshold is 1
     * @param nonce A nonce used to create a salt. Both the safe and module proxies share the same nonce.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @return addresses of the deployed module proxy and safe proxy.
     */
    function clone(
        address moduleSingletonAddress,
        address[] memory admins,
        uint256 nonce,
        bytes32 defaultTarget
    )
        public
        returns (address, address payable)
    {
        // Ensure there is at least one provided admin in the array
        if (admins.length == 0) {
            revert TooFewOwners();
        }
        // Generate a unique salt using the sender's address and the provided nonce
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        // 1. Deploy node management module proxy
        address moduleProxy = moduleSingletonAddress.cloneDeterministic(salt);

        // Temporarily replace one owner with the factory address
        address admin0 = admins[0];
        admins[0] = address(this);

        // Prepare safe initializer data
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

        // 2. Deploy Safe proxy
        SafeProxy safeProxy = SafeProxyFactory(SafeSuiteLib.SAFE_SafeProxyFactory_ADDRESS).createProxyWithNonce(
            SafeSuiteLib.SAFE_Safe_ADDRESS, safeInitializer, nonce
        );
        address payable safeProxyAddr = payable(address(safeProxy));

        // Add Safe and multisend to the module, then transfer ownership to the module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(address(safeProxy), SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS, defaultTarget)
        );
        moduleProxy.functionCall(moduleInitializer);

        // Enable the node management module
        bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
        prepareSafeTx(Safe(safeProxyAddr), 0, enableModuleData);
        // Swap owner for Safe proxy
        bytes memory swapOwnerData =
            abi.encodeWithSignature("swapOwner(address,address,address)", SENTINEL_OWNERS, address(this), admin0);
        prepareSafeTx(Safe(safeProxyAddr), 1, swapOwnerData);

        emit NewHoprNodeStakeModule(moduleSingletonAddress, moduleProxy);
        emit NewHoprNodeStakeSafe(address(safeProxy));
        return (moduleProxy, safeProxyAddr);
    }

    /**
     * @dev Prepares and executes a transaction on the safe contract.
     * @param safe The address of the safe contract.
     * @param nonce The nonce of the transaction.
     * @param data The data payload for the transaction.
     */
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

    /**
     * @dev Predicts the deterministic address that would result from deploying a contract instance with a given
     * implementation and salt.
     * @param implementation The address of the contract's implementation.
     * @param salt A unique value used to compute the deterministic address.
     * @return predicted The predicted address that the contract instance would have if deployed with the provided
     * implementation and salt.
     */
    function predictDeterministicAddress(
        address implementation,
        bytes32 salt
    )
        public
        view
        returns (address predicted)
    {
        return implementation.predictDeterministicAddress(salt);
    }
}
