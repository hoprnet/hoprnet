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
import { TargetUtils, Target } from "../utils/TargetUtils.sol";
import { IERC777Recipient } from "openzeppelin-contracts-5.4.0/interfaces/IERC777Recipient.sol";
import { IERC1820Registry } from "openzeppelin-contracts-5.4.0/interfaces/IERC1820Registry.sol";
import { IERC20, SafeERC20 } from "openzeppelin-contracts-5.4.0/token/ERC20/utils/SafeERC20.sol";

abstract contract HoprNodeStakeFactoryEvents {
    // Emit when a new module implementation is set
    event HoprNodeStakeModuleUpdated(address moduleImplementation);
    // Emit when a new module is created
    event NewHoprNodeStakeModule(address instance);
    // Emit when a new safe proxy is created
    event NewHoprNodeStakeSafe(address instance);
    // Emit when a new safe library address is set
    event HoprNodeStakeSafeLibUpdated(HoprNodeStakeFactory.SafeLibAddress safeLibAddresses);
    // Emit when a new default HOPR network is set
    event HoprNodeStakeHoprNetworkUpdated(HoprNodeStakeFactory.HoprNetwork hoprNetwork);
    // Emit when a new module is created for a (safe) wallet
    event NewHoprNodeStakeModuleForSafe(address module, address safe);
}

interface IHoprNodeStakeFactory {
    function deployModule(address safeProxyAddr, bytes32 defaultTarget, uint256 nonce) external returns (address moduleProxy);
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
contract HoprNodeStakeFactory is HoprNodeStakeFactoryEvents, Ownable2Step, IERC777Recipient, IHoprNodeStakeFactory {
    using TargetUtils for Target;
    using SafeERC20 for IERC20;

    // Error indicating that there are too few owners provided
    error TooFewOwners();
    // Error when providing the StakeFactory contract address as an owner
    error InvalidOwner();
    // Error when receiving tokens from an unauthorized token contract
    error UnauthorizedToken();
    // Error when the contract is not the token recipient
    error NotTokenRecipient();
    // Error when an invalid function selector is provided in userData
    error InvalidFunctionSelector();

    struct SafeLibAddress {
        address safeAddress;
        address safeProxyFactoryAddress;
        address fallbackHandlerAddress;
        address multiSendAddress;
    }

    struct HoprNetwork {
        address tokenAddress;
        uint256 defaultTokenAllowance;
        bytes32 defaultAnnouncementTarget;
    }

    // A sentinel address that serves as the start pointer of the owner linked list used in the OwnerManager of
    // safe-contracts
    address internal constant SENTINEL_OWNERS = address(0x1);
    // The address of the ERC1820 registry contract
    address internal constant ERC1820_ADDRESS = 0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24;
    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // function identifier from keccak256("_deploySafeAndModule(uint256,bytes32,address,address,uint256,address[])"))
    bytes32 public constant DEPLOYSAFEMODULE_FUNCTION_IDENTIFIER = 0xdd24c144db91d1bc600aac99393baf8f8c664ba461188f057e37f2c37b962b45;
    // function identifier from (keccak256("_deploySafeAndModuleAndIncludeNodes(uint256,bytes32,address,address,uint256,address[])"))
    bytes32 public constant DEPLOYSAFEANDMODULEANDINCLUDENODES_IDENTIFIER = 0x0105b97dcdf19d454ebe36f91ed516c2b90ee79f4a46af96a0138c1f5403c1cc;

    // Encoded address of the contract's approver, used for EIP-1271 signature verification
    bytes32 internal immutable r;
    // The singleton contract address of the HOPR node management module, as defined in the `HoprNodeManagementModule`
    address public moduleSingletonAddress;
    // Signature of the approved hash used for EIP-1271 signature verification
    bytes internal approvalHashSig;

    // Safe library addresses. Defaults to SafeSuiteLibV141 constants, but can be updated by the owner
    SafeLibAddress public safeLibAddresses = SafeLibAddress({
        safeAddress: SafeSuiteLibV141.SAFE_SafeL2_ADDRESS,
        safeProxyFactoryAddress: SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS,
        fallbackHandlerAddress: SafeSuiteLibV150.SAFE_CompatibilityFallbackHandler_ADDRESS,
        multiSendAddress: SafeSuiteLibV141.SAFE_MultiSend_ADDRESS
    });

    HoprNetwork public defaultHoprNetwork = HoprNetwork({
        tokenAddress: 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1,   // wxHOPR token address on Gnosis chain
        defaultTokenAllowance: 1000 ether,
        defaultAnnouncementTarget: bytes32(0)
    });

    modifier validateAdmins(address[] memory admins) {
        if (admins.length == 0) {
            revert TooFewOwners();
        }
        for (uint256 i = 0; i < admins.length; ++i) {
            if (admins[i] == address(this)) {
                revert InvalidOwner();
            }
        }
        _;
    }

    /**
     * @dev Constructor function to initialize contract state.
     * Initializes the encoded address of the contract's approver and the approved hash signature.
     */
    constructor(address _moduleSingletonAddress, address _announcementAddress, address initialOwner) Ownable(initialOwner) {
        // Encode the contract's address to be used in EIP-1271 signature verification
        r = bytes32(uint256(uint160(address(this))));
        // Encode the EIP-1271 contract signature for approval hash verification
        approvalHashSig = abi.encodePacked(abi.encode(r, bytes32(0)), bytes1(hex"01"));
        // Set the module singleton address
        _updateModuleSingletonAddress(_moduleSingletonAddress);
        // Register as an ERC777 token recipient
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
        // Set the initial announcement target
        defaultHoprNetwork.defaultAnnouncementTarget = bytes32(uint256(uint160(_announcementAddress))) << 96 | bytes32(uint256(0x010003000000000000000000));

        // Set the initial Safe library addresses
        emit HoprNodeStakeSafeLibUpdated(safeLibAddresses);
        // Set the initial HOPR network
        emit HoprNodeStakeHoprNetworkUpdated(defaultHoprNetwork);
    }

    /**
     * @dev Allows the contract owner to reclaim ERC20 tokens sent to the contract.
     * @param token The address of the ERC20 token contract.
     */
    function reclaimErc20(address token) external onlyOwner {
        IERC20(token).safeTransfer(owner(), IERC20(token).balanceOf(address(this)));
    }

    /**
     * @dev Handles the receipt of ERC777 tokens.
     * This function is called by the token contract when tokens are sent to this contract.
     * It deploys a Safe and a module proxy, and transfers the received tokens to the Safe.
     * @param from The address which sent the tokens.
     * @param to The address which received the tokens (should be this contract).
     * @param amount The amount of tokens received.
     * @param userData Additional data sent with the transfer, expected to contain caller, nonce, defaultTarget, and admins.
     */
    function tokensReceived(
        address, // operator not needed
        address from,
        address to,
        uint256 amount,
        bytes calldata userData,
        bytes calldata // operatorData not needed
    )
        external
        override
    {
        // Ensure that the tokens are sent to this contract
        require(to == address(this), NotTokenRecipient());
        // Ensure that the token sender is the default HOPR token
        require(msg.sender == defaultHoprNetwork.tokenAddress, UnauthorizedToken());

        // Decode userData to extract caller, nonce, defaultTarget, and admins
        (bytes32 functionIdentifier, uint256 nonce, bytes32 defaultTarget, address[] memory admins) = abi.decode(userData, (bytes32, uint256, bytes32, address[]));

        address payable safeProxy;
        // Ensure the function selector matches the expected value for this operation
        if (functionIdentifier == DEPLOYSAFEMODULE_FUNCTION_IDENTIFIER) {
            // Deploy Safe and module proxies
            (, safeProxy) = _deploySafeAndModule(nonce, defaultTarget, from, defaultHoprNetwork.tokenAddress, defaultHoprNetwork.defaultTokenAllowance, admins);
        } else if (functionIdentifier == DEPLOYSAFEANDMODULEANDINCLUDENODES_IDENTIFIER) {
            // Deploy Safe and module proxies using the clone function
            (, safeProxy) = _deploySafeAndModuleAndIncludeNodes(nonce, defaultTarget, from, defaultHoprNetwork.tokenAddress, defaultHoprNetwork.defaultTokenAllowance, admins);
        } else {
            revert InvalidFunctionSelector();

        }

        // Transfer the received tokens to the Safe proxy
        IERC20(defaultHoprNetwork.tokenAddress).safeTransfer(safeProxy, amount);
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
     * @dev Updates the default HOPR network configuration.
     * Can only be called by the contract owner.
     * @param _newHoprNetwork The new HOPR network configuration.
     */
    function updateHoprNetwork(HoprNetwork memory _newHoprNetwork) public onlyOwner {
        _updateHoprNetwork(_newHoprNetwork);
    }

    /**
     * @dev Updates the ERC1820 implementer for the contract.
     * Can only be called by the contract owner.
     * @param _newImplementer The new implementer address to be set in the ERC1820 registry.
     */
    function updateErc1820Implementer(address _newImplementer) public onlyOwner {
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, _newImplementer);
    }

    /**
     * @dev Deploys a 1-of-n Safe proxy and a module proxy for HOPR node management.
     * The Safe proxy is initialized with the provided list of admin addresses, and the module proxy
     * is initialized with the Safe proxy address, MultiSend address, and default target.
     * Module proxy is an ERC1967Proxy that follows UUPS pattern.
     * @notice This function has a compatible interface with the previous factory contract.
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
        return _deploySafeAndModule(nonce, defaultTarget, msg.sender, defaultHoprNetwork.tokenAddress, defaultHoprNetwork.defaultTokenAllowance, admins);
    }

    /**
     * @dev Deploys a module proxy for HOPR node management using CREATE2, from a deployed Safe proxy.
     * The module proxy is initialized with the Safe proxy address as owner, MultiSend address, and default target.
     * Module proxy is an ERC1967Proxy that follows UUPS pattern.
     * @notice Any wallet could call this function to deploy a module for an existing Safe, but the module
     * will only be usable if the Safe owner enables it.
     * @param safeProxyAddr The address of the Safe proxy that will be linked to the module.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @param nonce A nonce used to create a salt. Both the safe and module proxies share the same nonce.
     * @return moduleProxy The address of the deployed module proxy.
     */
    function deployModule(address safeProxyAddr, bytes32 defaultTarget, uint256 nonce) public returns (address moduleProxy) {
        moduleProxy = _deployModule(safeProxyAddr, defaultTarget, safeProxyAddr, nonce);
        emit NewHoprNodeStakeModuleForSafe(moduleProxy, safeProxyAddr);
    }

    /**
     * @dev Internal function to deploy a 1-of-n Safe proxy and a module proxy for HOPR node management.
     * The Safe proxy is initialized with the provided list of admin addresses, and the module proxy
     * is initialized with the Safe proxy address, MultiSend address, and default target.
     * Module proxy is an ERC1967Proxy that follows UUPS pattern.
     * @param admins The list of owners for the Safe proxy. The multisig threshold is 1
     * @param nonce A nonce used to create a salt. Both the safe and module proxies share the same nonce.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @return addresses of the deployed module proxy and safe proxy.
     */
    function _deploySafeAndModule(
        uint256 nonce,
        bytes32 defaultTarget,
        address caller,
        address tokenAddress,
        uint256 approveAmount,
        address[] memory admins
    )
        internal
        validateAdmins(admins)
        returns (address, address payable)
    {
        // Temporarily replace one owner with the factory address
        assembly {
            let len := mload(admins)
            mstore(admins, add(len, 1))
            mstore(add(admins, mul(0x20, add(len, 1))), address())
        }

        // 1. Prepare safe initializer data
        address payable safeProxyAddr = _deploySafe(admins, nonce);

        // 2. Prepare module initializer data
       address moduleProxy = _deployModule(safeProxyAddr, defaultTarget, caller, nonce);

        // 3. Send safe transactions
        {
            // - Nonce 0: set ERC777 token recipient implementer to the safe itself
            bytes memory setInterfaceData = abi.encodeWithSignature(
                "setInterfaceImplementer(address,bytes32,address)",
                safeProxyAddr,
                keccak256("ERC777TokensRecipient"),
                safeProxyAddr
            );
            _prepareSafeTx(safeProxyAddr, ERC1820_ADDRESS, setInterfaceData);
            // - Nonce 1: Enable the node management module
            bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
            _prepareSafeTx(safeProxyAddr, enableModuleData);
            // - Nonce 2: Approve the channels contract to spend HOPR tokens on behalf of the safe
            bytes memory approveData = abi.encodeWithSignature(
                "approve(address,uint256)",
                Target.wrap(uint256(defaultTarget)).getTargetAddress(), // channelAddress obtained from the default target
                approveAmount
            );
            _prepareSafeTx(safeProxyAddr, tokenAddress, approveData);
            // - Nonce 3: Renounce ownership from the safe
            bytes memory swapOwnerData =
                abi.encodeWithSignature("removeOwner(address,address,uint256)", admins[admins.length - 2], address(this), 1);
            _prepareSafeTx(safeProxyAddr, swapOwnerData);
        }

        return (moduleProxy, safeProxyAddr);
    }


    function _deploySafeAndModuleAndIncludeNodes(
        uint256 nonce,
        bytes32 defaultTarget,
        address caller,
        address tokenAddress,
        uint256 approveAmount,
        address[] memory admins
    )
        internal
        validateAdmins(admins)
        returns (address, address payable)
    {
        // Temporarily replace one owner with the factory address
        assembly {
            let len := mload(admins)
            mstore(admins, add(len, 1))
            mstore(add(admins, mul(0x20, add(len, 1))), address())
        }

        // 1. Prepare safe initializer data
        address payable safeProxyAddr = _deploySafe(admins, nonce);

        // 2. Prepare module initializer data
       address moduleProxy = _deployModule(safeProxyAddr, defaultTarget, caller, nonce);

        // 3. Send safe transactions
        {
            // - Nonce 0: set ERC777 token recipient implementer to the safe itself
            bytes memory setInterfaceData = abi.encodeWithSignature(
                "setInterfaceImplementer(address,bytes32,address)",
                safeProxyAddr,
                keccak256("ERC777TokensRecipient"),
                safeProxyAddr
            );
            _prepareSafeTx(safeProxyAddr, ERC1820_ADDRESS, setInterfaceData);
            // - Nonce 1: Enable the node management module
            bytes memory enableModuleData = abi.encodeWithSignature("enableModule(address)", moduleProxy);
            _prepareSafeTx(safeProxyAddr, enableModuleData);
            // - Nonce 2: Approve the channels contract to spend HOPR tokens on behalf of the safe
            bytes memory approveData = abi.encodeWithSignature(
                "approve(address,uint256)",
                Target.wrap(uint256(defaultTarget)).getTargetAddress(), // channelAddress obtained from the default target
                approveAmount
            );
            _prepareSafeTx(safeProxyAddr, tokenAddress, approveData);
            // revert to normal admins array length
            assembly {
                let len := mload(admins)
                mstore(admins, sub(len, 1))
            }
            // - Nonce 3: Add the caller as a node in the module
            bytes memory addNodeData = abi.encodeWithSignature("includeNodes(address[])", admins);
            _prepareSafeTx(safeProxyAddr, moduleProxy, addNodeData);
            // - Nonce 4: Renounce ownership from the safe
            bytes memory swapOwnerData =
                abi.encodeWithSignature("removeOwner(address,address,uint256)", admins[admins.length - 1], address(this), 1);
            _prepareSafeTx(safeProxyAddr, swapOwnerData);
        }

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
     * @dev Updates the default HOPR network configuration and emits an event.
     * @param _newHoprNetwork The new HOPR network configuration.
     */
    function _updateHoprNetwork(HoprNetwork memory _newHoprNetwork) private {
        defaultHoprNetwork = _newHoprNetwork;
        emit HoprNodeStakeHoprNetworkUpdated(defaultHoprNetwork);
    }

    /**
     * @dev Prepares and executes a transaction on the safe contract.
     * @param safeAddress The address of the safe contract.
     * @param data The data payload for the transaction.
     */
    function _prepareSafeTx(address safeAddress, bytes memory data) private {
        Safe(payable(safeAddress)).execTransaction(
            safeAddress,                  // to address
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
     * @param safeAddress The address of the safe contract.
     * @param to The destination address of the transaction.
     * @param data The data payload for the transaction.
     */
    function _prepareSafeTx(address safeAddress, address to, bytes memory data) private {
        Safe(payable(safeAddress)).execTransaction(
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
     * @dev Deploys a Safe proxy with the provided admin addresses and nonce.
     * @param admins The list of owners for the Safe proxy. The multisig threshold is 1
     * @param nonce A nonce used to create a salt.
     * @return safeProxyAddr The address of the deployed Safe proxy.
     */
    function _deploySafe(address[] memory admins, uint256 nonce) private returns (address payable safeProxyAddr) {
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

        // Deploy Safe proxy
        SafeProxy safeProxy = SafeProxyFactory(safeLibAddresses.safeProxyFactoryAddress).createProxyWithNonce(
            safeLibAddresses.safeAddress, safeInitializer, nonce
        );
        safeProxyAddr = payable(address(safeProxy));
        emit NewHoprNodeStakeSafe(safeProxyAddr);
    }

    /**
     * @dev Deploys a module proxy for HOPR node management using CREATE2.
     * The module proxy is initialized with the Safe proxy address, MultiSend address, and default target.
     * Module proxy is an ERC1967Proxy that follows UUPS pattern.
     * @param safeProxyAddr The address of the Safe proxy that will be linked to the module.
     * @param defaultTarget The default target (refer to TargetUtils.sol) for the current HoprChannels (and HoprToken)
     * contract.
     * @param caller The address of the deployer.
     * @param nonce A nonce used to create a salt. Both the safe and module proxies share the same nonce.
     * @return moduleProxy The address of the deployed module proxy.
     */
    function _deployModule(address safeProxyAddr, bytes32 defaultTarget, address caller, uint256 nonce) private returns (address moduleProxy) {
         // Add Safe and multisend to the module, then transfer ownership to the module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(safeProxyAddr, safeLibAddresses.multiSendAddress, defaultHoprNetwork.defaultAnnouncementTarget, defaultTarget)
        );

        // Generate a unique salt using the sender's address and the provided nonce
        /// forge-lint: disable-next-line(asm-keccak256)
        bytes32 salt = keccak256(abi.encodePacked(caller, nonce));

        // Deploy module proxy (ERC1967Proxy) with CREATE2
        moduleProxy = Create2.deploy(0, salt, abi.encodePacked(
            type(ERC1967Proxy).creationCode, 
            abi.encode(
                moduleSingletonAddress,
                moduleInitializer
            )
        ));
        emit NewHoprNodeStakeModule(moduleProxy);
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
        // forge-lint: disable-start(asm-keccak256)
        bytes32 bytecodeHash = keccak256(abi.encodePacked(
            type(ERC1967Proxy).creationCode, 
            abi.encode(
                moduleSingletonAddress,
                abi.encodeWithSignature(
                    "initialize(bytes)",
                    abi.encode(safe, safeLibAddresses.multiSendAddress, defaultHoprNetwork.defaultAnnouncementTarget, defaultTarget)
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
        // forge-lint: disable-start(asm-keccak256)
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
