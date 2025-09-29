// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import { SafeSuiteLibV141 } from "../utils/SafeSuiteLibV141.sol";
import { SafeSuiteLibV150 } from "../utils/SafeSuiteLibV150.sol";
import { SafeProxy } from "safe-contracts-1.4.1/proxies/SafeProxy.sol";
import { SafeProxyFactory } from "safe-contracts-1.4.1/proxies/SafeProxyFactory.sol";
import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { Enum } from "safe-contracts-1.4.1/common/Enum.sol";
import { Ownable2Step } from "openzeppelin-contracts-5.4.0/access/Ownable2Step.sol";
import { Ownable } from "openzeppelin-contracts-5.4.0/access/Ownable2Step.sol";
import { Create2 } from "openzeppelin-contracts-5.4.0/utils/Create2.sol";
import { ERC1967Proxy } from "openzeppelin-contracts-5.4.0/proxy/ERC1967/ERC1967Proxy.sol";

abstract contract HoprNodeStakeFactoryEvents {
    // Emit when a new module implementation is set
    event HoprNodeStakeModuleUpdated(address moduleImplementation);
    // Emit when a new module is created
    event NewHoprNodeStakeModule(address instance);
    // Emit when a new safe proxy is created
    event NewHoprNodeStakeSafe(address instance);
    // Emit when a new safe library address is set
    event HoprNodeStakeSafeLibUpdated(HoprNodeStakeFactory.SafeLibAddress safeLibAddresses);
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
    // Error indicating that there are too few owners provided
    error TooFewOwners();
    // Error when providing the StakeFactory contract address as an owner
    error InvalidOwner();

    struct SafeLibAddress {
        address safeAddress;
        address safeProxyFactoryAddress;
        address fallbackHandlerAddress;
        address multiSendAddress;
    }

    // A sentinel address that serves as the start pointer of the owner linked list used in the OwnerManager of
    // safe-contracts
    address internal constant SENTINEL_OWNERS = address(0x1);

    // The address of the ERC1820 registry contract
    address internal constant ERC1820_ADDRESS = 0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24;

    // Encoded address of the contract's approver, used for EIP-1271 signature verification
    bytes32 internal immutable r;

    // The singleton contract address of the HOPR node management module, as defined in the `HoprNodeManagementModule`
    address public moduleSingletonAddress;

    // Signature of the approved hash used for EIP-1271 signature verification
    bytes internal approvalHashSig;

    // Safe library addresses. Defaults to SafeSuiteLibV141 constants, but can be updated by the owner
    SafeLibAddress public safeLibAddresses = SafeLibAddress({
        safeAddress: SafeSuiteLibV141.SAFE_Safe_ADDRESS,
        safeProxyFactoryAddress: SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS,
        fallbackHandlerAddress: SafeSuiteLibV150.SAFE_CompatibilityFallbackHandler_ADDRESS,
        multiSendAddress: SafeSuiteLibV141.SAFE_MultiSend_ADDRESS
    });

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

        // Set the initial Safe library addresses
        emit HoprNodeStakeSafeLibUpdated(safeLibAddresses);
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
     * @dev Updates the Safe library addresses. Can only be called by the contract owner.
     * @param _newSafeLibAddresses The new Safe library addresses.
     */
    function updateSafeLibAddress(SafeLibAddress memory _newSafeLibAddresses) public onlyOwner {
        _updateSafeLibAddress(_newSafeLibAddresses);
    }

    /**
     * @dev Deploys a 1-of-n Safe proxy and a module proxy for HOPR node management.
     * The Safe proxy is initialized with the provided list of admin addresses, and the module proxy
     * is initialized with the Safe proxy address, MultiSend address, and default target.
     * Module proxy is an ERC1967Proxy that follows UUPS pattern.
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
        // 0. Validate inputs
        // Ensure there is at least one provided admin in the array
        if (admins.length == 0) {
            revert TooFewOwners();
        }
        for (uint256 i = 0; i < admins.length; ++i) {
            if (admins[i] == address(this)) {
                revert InvalidOwner();
            }
        }

        // Temporarily replace one owner with the factory address
        assembly {
            let len := mload(admins)
            mstore(admins, add(len, 1))
            mstore(add(admins, mul(0x20, add(len, 1))), address())
        }

        // 1. Prepare safe initializer data
        bytes memory safeInitializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            safeLibAddresses.fallbackHandlerAddress,
            address(0),
            0,
            address(0)
        );

        // 2. Deploy Safe proxy
        SafeProxy safeProxy = SafeProxyFactory(safeLibAddresses.safeProxyFactoryAddress).createProxyWithNonce(
            safeLibAddresses.safeAddress, safeInitializer, nonce
        );
        address payable safeProxyAddr = payable(address(safeProxy));

        // 3. Prepare module initializer data
        // Add Safe and multisend to the module, then transfer ownership to the module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(address(safeProxy), safeLibAddresses.multiSendAddress, defaultTarget)
        );

        // Generate a unique salt using the sender's address and the provided nonce
        /// forge-lint: disable-next-line(asm-keccak256)
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        // 4. Deploy module proxy (ERC1967Proxy) with CREATE2
        address moduleProxy = Create2.deploy(0, salt, abi.encodePacked(
            type(ERC1967Proxy).creationCode, 
            abi.encode(
                moduleSingletonAddress,
                moduleInitializer
            )
        ));

        // set ERC777 token recipient implementer to the safe itself
        bytes memory setInterfaceData = abi.encodeWithSignature(
            "setInterfaceImplementer(address,bytes32,address)",
            safeProxyAddr,
            keccak256("ERC777TokensRecipient"),
            safeProxyAddr
        );
        _prepareSafeTx(Safe(safeProxyAddr), ERC1820_ADDRESS, setInterfaceData);
        // Enable the node management module
        bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
        _prepareSafeTx(Safe(safeProxyAddr), enableModuleData);
        // Renonce ownership from the safe
        bytes memory swapOwnerData =
            abi.encodeWithSignature("removeOwner(address,address,uint256)", admins[admins.length - 2], address(this), 1);
        _prepareSafeTx(Safe(safeProxyAddr), swapOwnerData);


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
     * @dev Updates the Safe library addresses and emits an event.
     * @param _newSafeLibAddresses The new Safe library addresses.
     */
    function _updateSafeLibAddress(SafeLibAddress memory _newSafeLibAddresses) private {
        safeLibAddresses = _newSafeLibAddresses;
        emit HoprNodeStakeSafeLibUpdated(safeLibAddresses);
    }

    /**
     * @dev Prepares and executes a transaction on the safe contract.
     * @param safe The address of the safe contract.
     * @param data The data payload for the transaction.
     */
    function _prepareSafeTx(Safe safe, bytes memory data) private {
        safe.execTransaction(
            address(safe),                  // to address
            0,                              // value
            data,                           // data
            Enum.Operation.Call,            // operation
            0,                              // safeTxGas
            0,                              // baseGas
            0,                              // gasPrice
            address(0),                     // gasToken
            payable(address(msg.sender)),   // refundReceiver
            approvalHashSig                 // signature
        );
    }

    /**
     * @dev Prepares and executes a transaction on the safe contract.
     * @param safe The address of the safe contract.
     * @param to The destination address of the transaction.
     * @param data The data payload for the transaction.
     */
    function _prepareSafeTx(Safe safe, address to, bytes memory data) private {
        safe.execTransaction(
            to,                             // to address
            0,                              // value
            data,                           // data
            Enum.Operation.Call,            // operation
            0,                              // safeTxGas
            0,                              // baseGas
            0,                              // gasPrice
            address(0),                     // gasToken
            payable(address(msg.sender)),   // refundReceiver
            approvalHashSig                 // signature
        );
    }

    /**
     * @dev Predicts the deterministic address that would result from deploying a contract instance with a given
     * implementation and salt.
     * @param salt A unique value used to compute the deterministic address.
     * @return predicted The predicted address that the contract instance would have if deployed with the provided
     * implementation and salt.
     */
    function predictModuleAddress(bytes32 salt, address safe, bytes32 defaultTarget)
        public
        view
        returns (address predicted)
    {
        bytes32 bytecodeHash = keccak256(abi.encodePacked(
            type(ERC1967Proxy).creationCode, 
            abi.encode(
                moduleSingletonAddress,
                abi.encodeWithSignature(
                    "initialize(bytes)",
                    abi.encode(safe, safeLibAddresses.multiSendAddress, defaultTarget)
                )
            )
        ));

        return Create2.computeAddress(salt, bytecodeHash);
    }
    /**
     * @dev Predicts the deterministic address of a module proxy deployed by this factory for a given deployer and nonce
     *      using the CREATE2 opcode.
     *      new_contract_address = keccak256(0xff ++ deployer ++ salt ++ keccak256(init_code))
     * @param caller The address of the deployer.
     * @param nonce A unique value used to compute the deterministic address.
     * @param safe The address of the Safe proxy that will be linked to the module.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @return predicted The predicted address that the module proxy would have if deployed by the specified deployer
     * with the provided nonce.
     */
    function predictModuleAddress(address caller, uint256 nonce, address safe, bytes32 defaultTarget)
        public
        view
        returns (address predicted)
    {
        bytes32 salt = keccak256(abi.encodePacked(caller, nonce));
        return predictModuleAddress(salt, safe, defaultTarget);
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
    function predictSafeAddress(address[] memory admins, uint256 nonce) public view returns (address predicted) {
        // Temporarily replace one owner with the factory address
        assembly {
            let len := mload(admins)
            mstore(admins, add(len, 1))
            mstore(add(admins, mul(0x20, add(len, 1))), address())
        }

        bytes memory initializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            safeLibAddresses.fallbackHandlerAddress,
            address(0),
            0,
            address(0)
        );
        // forge-lint: disable-start(asm-keccak256)
        bytes32 salt = keccak256(abi.encodePacked(keccak256(initializer), nonce));
        bytes32 predictedHash = keccak256(
            abi.encodePacked(
                bytes1(0xff),
                address(safeLibAddresses.safeProxyFactoryAddress),
                salt,
                keccak256(
                    abi.encodePacked(
                        SafeProxyFactory(safeLibAddresses.safeProxyFactoryAddress).proxyCreationCode(),
                        uint256(uint160(safeLibAddresses.safeAddress))
                    )
                )
            )
        );
        // forge-lint: disable-end(asm-keccak256)
        return address(uint160(uint256(predictedHash)));
    }
}
