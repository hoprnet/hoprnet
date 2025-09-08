// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable-4.9.2/proxy/ClonesUpgradeable.sol";
import { Address } from "openzeppelin-contracts-4.9.2/utils/Address.sol";
import { SafeSuiteLibV141 } from "../utils/SafeSuiteLibV141.sol";
import { SafeProxy } from "safe-contracts-1.4.1/proxies/SafeProxy.sol";
import { SafeProxyFactory } from "safe-contracts-1.4.1/proxies/SafeProxyFactory.sol";
import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { Enum } from "safe-contracts-1.4.1/common/Enum.sol";
import { Ownable2Step } from "openzeppelin-contracts-5.4.0/access/Ownable2Step.sol";
import { Ownable } from "openzeppelin-contracts-5.4.0/access/Ownable2Step.sol";

abstract contract HoprNodeStakeFactoryEvents {
    event HoprNodeStakeModuleUpdated(address moduleImplementation); // Emit when a new module implementation is set
    event NewHoprNodeStakeModule(address instance); // Emit when a new module is created
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
contract HoprNodeStakeFactory is HoprNodeStakeFactoryEvents, Ownable2Step {
    using Address for address;
    using ClonesUpgradeable for address;

    // A sentinel address that serves as the start pointer of the owner linked list used in the OwnerManager of
    // safe-contracts
    address internal constant SENTINEL_OWNERS = address(0x1);

    // Encoded address of the contract's approver, used for EIP-1271 signature verification
    bytes32 internal immutable r;

    // The singleton contract address of the HOPR node management module, as defined in the `HoprNodeManagementModule`
    address public moduleSingletonAddress;

    // Signature of the approved hash used for EIP-1271 signature verification
    bytes internal approvalHashSig;

    // Error indicating that there are too few owners provided
    error TooFewOwners();
    // Error when providing the StakeFactory contract address as an owner
    error InvalidOwner();

    /**
     * @dev Constructor function to initialize contract state.
     * Initializes the encoded address of the contract's approver and the approved hash signature.
     */
    constructor(address _moduleSingletonAddress, address initialOwner) Ownable(initialOwner) {
        // Encode the contract's address to be used in EIP-1271 signature verification
        r = bytes32(uint256(uint160(address(this))));

        // Encode the EIP-1271 contract signature for approval hash verification
        approvalHashSig = abi.encodePacked(abi.encode(r, bytes32(0)), bytes1(hex"01"));

        // Set the module singleton address
        _updateModuleSingletonAddress(_moduleSingletonAddress);
    }

    /**
     * @dev Returns the version of Safe deployments
     */
    function safeVersion() public pure returns (string memory) {
        return SafeSuiteLibV141.SAFE_VERSION;
    }

    /**
     * @dev Updates the module singleton address. Can only be called by the contract owner.
     * @param _newModuleSingletonAddress The new address of the module singleton.
     */
    function updateModuleSingletonAddress(address _newModuleSingletonAddress) public onlyOwner {
        _updateModuleSingletonAddress(_newModuleSingletonAddress);
    }

    /**
     * @dev Deploys a 1-of-n Safe proxy and a module proxy for HOPR node management.
     * @param admins The list of owners for the Safe proxy. The multisig threshold is 1
     * @param nonce A nonce used to create a salt. Both the safe and module proxies share the same nonce.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @return addresses of the deployed module proxy and safe proxy.
     */
    function clone(
        uint256 nonce,
        bytes32 defaultTarget,
        address[] memory admins
    )
        public
        returns (address, address payable)
    {
        // Ensure there is at least one provided admin in the array
        if (admins.length == 0) {
            revert TooFewOwners();
        }
        for (uint256 i = 0; i < admins.length; ++i) {
            if (admins[i] == address(this)) {
                revert InvalidOwner();
            }
        }
        // Generate a unique salt using the sender's address and the provided nonce
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        // 1. Deploy node management module proxy
        address moduleProxy = moduleSingletonAddress.cloneDeterministic(salt);

        // Temporarily replace one owner with the factory address
        assembly {
            let len := mload(admins)
            mstore(admins, add(len, 1))
            mstore(add(admins, mul(0x20, add(len, 1))), address())
        }

        // Prepare safe initializer data
        bytes memory safeInitializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            SafeSuiteLibV141.SAFE_CompatibilityFallbackHandler_ADDRESS,
            address(0),
            0,
            address(0)
        );

        // 2. Deploy Safe proxy
        SafeProxy safeProxy = SafeProxyFactory(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS).createProxyWithNonce(
            SafeSuiteLibV141.SAFE_Safe_ADDRESS, safeInitializer, nonce
        );
        address payable safeProxyAddr = payable(address(safeProxy));

        // Add Safe and multisend to the module, then transfer ownership to the module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(address(safeProxy), SafeSuiteLibV141.SAFE_MultiSendCallOnly_ADDRESS, defaultTarget)
        );
        moduleProxy.functionCall(moduleInitializer);

        // Enable the node management module
        bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
        prepareSafeTx(Safe(safeProxyAddr), 0, enableModuleData);
        // Renonce ownership from the safe
        bytes memory swapOwnerData =
            abi.encodeWithSignature("removeOwner(address,address,uint256)", admins[admins.length - 2], address(this), 1);
        prepareSafeTx(Safe(safeProxyAddr), 1, swapOwnerData);

        emit NewHoprNodeStakeModule(moduleProxy);
        emit NewHoprNodeStakeSafe(address(safeProxy));
        return (moduleProxy, safeProxyAddr);
    }

    /**
     * @dev Updates the module singleton address and emits an event.
     * @param _newModuleSingletonAddress The new address of the module singleton.
     */
    function _updateModuleSingletonAddress(address _newModuleSingletonAddress) private {
        moduleSingletonAddress = _newModuleSingletonAddress;
        emit HoprNodeStakeModuleUpdated(moduleSingletonAddress);
    }

    /**
     * @dev Prepares and executes a transaction on the safe contract.
     * @param safe The address of the safe contract.
     * @param nonce The nonce of the transaction.
     * @param data The data payload for the transaction.
     */
    function prepareSafeTx(Safe safe, uint256 nonce, bytes memory data) private {
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
     * @param salt A unique value used to compute the deterministic address.
     * @return predicted The predicted address that the contract instance would have if deployed with the provided
     * implementation and salt.
     */
    function predictModuleAddress(bytes32 salt)
        public
        view
        returns (address predicted)
    {
        return moduleSingletonAddress.predictDeterministicAddress(salt);
    }

    /**
     * @dev Predicts the deterministic address of a Safe proxy deployed by this factory for a given deployer and nonce
     *      using the CREATE2 opcode.
     *      new_contract_address = keccak256(0xff ++ deployer ++ salt ++ keccak256(init_code))
     * @param admins An array of admin addresses
     * @param nonce A unique value used to compute the deterministic address.
     * @return predicted The predicted address that the Safe proxy would have if deployed by the specified deployer
     * with the provided nonce.
     */
    function predictSafeAddress(address[] memory admins, uint256 nonce) public pure returns (address predicted) {
        bytes memory initializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            SafeSuiteLibV141.SAFE_CompatibilityFallbackHandler_ADDRESS,
            address(0),
            0,
            address(0)
        );
        bytes32 salt = keccak256(abi.encodePacked(keccak256(initializer), nonce));
        bytes32 predictedHash = keccak256(
            abi.encodePacked(
                bytes1(0xff),
                address(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS),
                salt,
                keccak256(
                    abi.encodePacked(
                        SafeProxyFactory(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS).proxyCreationCode(),
                        uint256(uint160(SafeSuiteLibV141.SAFE_Safe_ADDRESS))
                    )
                )
            )
        );
        return address(uint160(uint256(predictedHash)));
    }
}
