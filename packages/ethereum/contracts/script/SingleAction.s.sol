// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "./utils/NetworkConfig.s.sol";
import "./utils/BoostUtilsLib.sol";
import "../src/utils/TargetUtils.sol";

abstract contract Enum {
    enum Operation {
        Call,
        DelegateCall
    }
}

abstract contract ISafe {
    function getTransactionHash(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address refundReceiver,
        uint256 _nonce
    )
        public
        view
        virtual
        returns (bytes32);

    function execTransaction(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address payable refundReceiver,
        bytes memory signatures
    )
        public
        payable
        virtual
        returns (bool success);

    function nonce() public virtual returns (uint256);
}

abstract contract IFactory {
    function clone(
        address moduleSingletonAddress,
        address[] memory admins,
        uint256 nonce,
        bytes32 defaultTarget
    )
        public
        virtual
        returns (address, address payable);
}

/// Failed to read balance of a token contract
/// @param token token address.
error FailureInReadBalance(address token);

/**
 * @dev script to interact with contract(s) of a given environment where the msg.sender comes from the environment
 * variable `PRIVATE_KEY`
 * Private key of the caller must be saved under the environment variable `PRIVATE_KEY`
 * Wrapper of contracts (incl. NetworkRegistery, HoprStake) with detection of contract address per
 * network/environment_type
 */
contract SingleActionFromPrivateKeyScript is Test, NetworkConfig {
    using stdJson for string;
    using stdStorage for StdStorage;

    using BoostUtilsLib for address;
    using TargetUtils for Target;

    address msgSender;
    string[] private unregisteredIds;
    address[] private accounts;
    address[] private nodes;

    function getNetworkAndMsgSender() private {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();

        // 2. Get private key of caller
        // Set to default when it's in development environment (uint for
        // 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 privateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77_814_517_325_470_205_911_140_941_194_401_928_579_557_062_014_761_831_930_645_393_041_380_819_009_408
            : vm.envUint("PRIVATE_KEY");
        msgSender = vm.addr(privateKey);
        emit log_named_address("msgSender address", msgSender);
        vm.startBroadcast(privateKey);
    }

    /**
     * @dev create a safe proxy and moodule proxy.
     * Perform the following actions as the owner of safe:
     * - include nodes to the module
     * - approve token transfer
     * - add announcement contract as target
     * As manager of network registry, add nodes and safe to network registry
     * Perform the following actions, as deployer
     * - transfer some tokens to safe
     * - transfer some xDAI to nodes
     *
     * @notice Deployer is the single owner of safe
     * nonce is the current nonce of deployer account
     * Default fallback permission for module is to
     * 1. allow all data to Channels contract
     * 2. allow all data to Token contract
     * 3. allow nodes to send native tokens to itself
     *
     * Give Channels max allowance to transfer Token for safe
     *
     * Add node safes to network registry, as a manager
     * @param nodeAddresses array of node addresses to be added to the module
     * @param hoprTokenAmountInWei, The amount of HOPR tokens that recipient is desired to receive
     * @param nativeTokenAmountInWei The amount of native tokens that recipient is desired to receive
     */
    function expressSetupSafeModule(
        address[] memory nodeAddresses,
        uint256 hoprTokenAmountInWei,
        uint256 nativeTokenAmountInWei
    )
        external
        returns (address safe, address module)
    {
        // 1. get environment and msg.sender
        getNetworkAndMsgSender();

        // 2. prepare parameters for proxy deployment
        address[] memory admins = new address[](1);
        admins[0] = msgSender;
        /**
         * Array of capability permissions
         *     [
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultRedeemTicketSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // RESERVED
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultCloseIncomingChannelSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, //
         * defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, //
         * defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultApproveFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW  // defaultSendFunctionPermisson
         *     ]
         */
        CapabilityPermission[] memory defaultChannelsCapabilityPermissions = new CapabilityPermission[](9);
        for (uint256 i = 0; i < defaultChannelsCapabilityPermissions.length; i++) {
            defaultChannelsCapabilityPermissions[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        Target defaultModulePermission = TargetUtils.encodeDefaultPermissions(
            currentNetworkDetail.addresses.channelsContractAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultChannelsCapabilityPermissions
        );

        // 3. deploy two proxy instances
        (module, safe) = IFactory(currentNetworkDetail.addresses.nodeStakeV2FactoryAddress).clone(
            currentNetworkDetail.addresses.moduleImplementationAddress,
            admins,
            vm.getNonce(msgSender),
            bytes32(Target.unwrap(defaultModulePermission))
        );

        emit log_string(
            string(abi.encodePacked("--safeAddress ", vm.toString(safe), " --moduleAddress ", vm.toString(module)))
        );

        // 4. include nodes to the module, as an owner of safe
        includeNodesToModuleBySafe(nodeAddresses, safe, module);

        // 5. approve token transfer, as an owner of safe
        approveChannelsForTokenTransferBySafe(safe);

        // 6. add announcement contract as target, as an owner of safe
        addAllAllowedTargetToModuleBySafe(currentNetworkDetail.addresses.announcements, safe, module);
        // bytes memory
        vm.stopBroadcast();

        // 7. add nodes and safe to network registry, as a manager of network registry
        address[] memory stakingSafeAddresses = new address[](nodeAddresses.length);
        for (uint256 m = 0; m < nodeAddresses.length; m++) {
            stakingSafeAddresses[m] = safe;
        }
        _helperGetDeployerInternalKey();
        _registerNodes(stakingSafeAddresses, nodeAddresses);
        vm.stopBroadcast();

        // 8. transfer some tokens to safe
        transferOrMintHoprAndSendNativeToAmount(safe, hoprTokenAmountInWei, nativeTokenAmountInWei);

        // 9. transfer some xDAI to nodes
        for (uint256 n = 0; n < nodeAddresses.length; n++) {
            transferOrMintHoprAndSendNativeToAmount(safe, 0, nativeTokenAmountInWei);
        }
    }

    /**
     * @dev Given existing node(s), safe and module, migrate them to a different network
     * Perform the following actions as the owner of safe:
     * - scope new channel contract
     * - approve token transfer
     * - add announcement contract as target
     * As manager of network registry, add nodes and safe to network registry
     *
     * @notice Deployer is the single owner of safe
     * nonce is the current nonce of deployer account
     * Default fallback permission for module is to
     * 1. allow all data to Channels contract
     * 2. allow all data to Token contract
     * 3. allow nodes to send native tokens to itself
     *
     * Give Channels max allowance to transfer Token for safe
     *
     * Add node safes to network registry, as a manager
     * @param nodeAddresses array of node addresses to be added to the module
     * @param safe safe address of node
     * @param module module address of node
     */
    function migrateSafeModule(address[] memory nodeAddresses, address safe, address module) external {
        // 1. get environment and msg.sender
        getNetworkAndMsgSender();

        // 2. scope channel contract of the new network
        addNetworkChannelsTargetToModuleBySafe(safe, module);

        // 3. approve token transfer, as an owner of safe
        approveChannelsForTokenTransferBySafe(safe);

        // 4. add announcement contract as target, as an owner of safe
        addAllAllowedTargetToModuleBySafe(currentNetworkDetail.addresses.announcements, safe, module);
        // bytes memory
        vm.stopBroadcast();

        // 5. add nodes and safe to network registry, as a manager of network registry
        address[] memory stakingSafeAddresses = new address[](nodeAddresses.length);
        for (uint256 m = 0; m < nodeAddresses.length; m++) {
            stakingSafeAddresses[m] = safe;
        }
        _helperGetDeployerInternalKey();
        _registerNodes(stakingSafeAddresses, nodeAddresses);
        vm.stopBroadcast();
    }

    function includeNodesToModuleBySafe(address[] memory nodeAddresses, address safe, address module) public {
        // 1. get the msgSender if not set. This msgSender should be the owner of safe to execute the tx
        if (msgSender == address(0)) {
            // get environment and msg.sender
            getNetworkAndMsgSender();
        }

        // 2. prepare target permission data
        /**
         * Array of node permissions, where nothing is specified and falls back to the default
         *     [
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE
         *     ]
         */
        CapabilityPermission[] memory nodeDefaultPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < nodeDefaultPermission.length; i++) {
            nodeDefaultPermission[i] = CapabilityPermission.NONE;
        }
        // allow node to send native tokens to itself
        Target[] memory defaultNodeTargets = new Target[](nodeAddresses.length);
        for (uint256 j = 0; j < nodeAddresses.length; j++) {
            defaultNodeTargets[j] = TargetUtils.encodeDefaultPermissions(
                nodeAddresses[j], Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, nodeDefaultPermission
            );
        }

        // 4. include nodes to the module, as an owner of safe
        for (uint256 k = 0; k < nodeAddresses.length; k++) {
            // check if node is included in module
            (bool successReadIncluded, bytes memory returndataReadIncluded) =
                module.staticcall(abi.encodeWithSignature("isNode(address)", nodeAddresses[k]));
            if (!successReadIncluded) {
                revert("Cannot read isNode from module contract.");
            }
            bool included = abi.decode(returndataReadIncluded, (bool));
            if (!included) {
                bytes memory includeNodeData =
                    abi.encodeWithSignature("includeNode(uint256)", Target.unwrap(defaultNodeTargets[k]));
                uint256 safeNonce = ISafe(payable(safe)).nonce();
                _helperSignSafeTxAsOwner(ISafe(payable(safe)), module, safeNonce, includeNodeData);
            }
        }
    }

    function approveChannelsForTokenTransferBySafe(address safe) public {
        // 1. get the msgSender if not set. This msgSender should be the owner of safe to execute the tx
        if (msgSender == address(0)) {
            // get environment and msg.sender
            getNetworkAndMsgSender();
        }

        // 2. prepare data payload for approve
        bytes memory approveData = abi.encodeWithSignature(
            "approve(address,uint256)", currentNetworkDetail.addresses.channelsContractAddress, type(uint256).max
        );
        _helperSignSafeTxAsOwner(
            ISafe(payable(safe)),
            currentNetworkDetail.addresses.tokenContractAddress,
            ISafe(payable(safe)).nonce(),
            approveData
        );
    }

    /**
     * add an ALL_ALLOWED target to the module, by the safe
     * Abuse TOKEN type
     */
    function addAllAllowedTargetToModuleBySafe(address targetAddress, address safe, address module) public {
        // 1. get the msgSender if not set. This msgSender should be the owner of safe to execute the tx
        if (msgSender == address(0)) {
            // get environment and msg.sender
            getNetworkAndMsgSender();
        }

        // 2. prepare target permission data
        /**
         * Array of node permissions, where nothing is specified and falls back to the default
         *     [
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE,
         *       CapabilityPermission.NONE
         *     ]
         */
        CapabilityPermission[] memory defaultPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < defaultPermission.length; i++) {
            defaultPermission[i] = CapabilityPermission.NONE;
        }
        // abuse the fast return and assign target as 0
        Target target = TargetUtils.encodeDefaultPermissions(
            targetAddress, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultPermission
        );

        // 4. include the target to the module, as an owner of safe
        // check if target has been included in module.
        (bool successReadTryGetTarget, bytes memory returndataTryGetTarget) =
            module.staticcall(abi.encodeWithSignature("tryGetTarget(address)", targetAddress));
        if (!successReadTryGetTarget) {
            revert("Cannot read tryGetTarget from module contract.");
        }
        (bool included,) = abi.decode(returndataTryGetTarget, (bool, uint256));
        if (!included) {
            bytes memory scopeTargetData = abi.encodeWithSignature("scopeTargetToken(uint256)", Target.unwrap(target));
            uint256 safeNonce = ISafe(payable(safe)).nonce();

            _helperSignSafeTxAsOwner(ISafe(payable(safe)), module, safeNonce, scopeTargetData);
        }
    }

    /**
     * add an ALL_ALLOWED Channels target of the current network's channels contract to the module, by the safe
     */
    function addNetworkChannelsTargetToModuleBySafe(address safe, address module) public {
        // 1. get the msgSender if not set. This msgSender should be the owner of safe to execute the tx
        if (msgSender == address(0)) {
            // get environment and msg.sender
            getNetworkAndMsgSender();
        }
        address targetAddress = currentNetworkDetail.addresses.channelsContractAddress;
        // 2. scope channel contract
        /**
         * Array of capability permissions
         *     [
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultRedeemTicketSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // RESERVED
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultCloseIncomingChannelSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, //
         * defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, //
         * defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultApproveFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW  // defaultSendFunctionPermisson
         *     ]
         */
        CapabilityPermission[] memory channelsCapabilityPermissions = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsCapabilityPermissions.length; i++) {
            channelsCapabilityPermissions[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        Target target = TargetUtils.encodeDefaultPermissions(
            targetAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            channelsCapabilityPermissions
        );

        // 3. include the target to the module, as an owner of safe
        // check if target has been included in module.
        (bool successReadTryGetTarget, bytes memory returndataTryGetTarget) =
            module.staticcall(abi.encodeWithSignature("tryGetTarget(address)", targetAddress));
        if (!successReadTryGetTarget) {
            revert("Cannot read tryGetTarget from module contract.");
        }
        (bool included,) = abi.decode(returndataTryGetTarget, (bool, uint256));
        if (!included) {
            bytes memory scopeTargetData =
                abi.encodeWithSignature("scopeTargetChannels(uint256)", Target.unwrap(target));
            uint256 safeNonce = ISafe(payable(safe)).nonce();

            _helperSignSafeTxAsOwner(ISafe(payable(safe)), module, safeNonce, scopeTargetData);
        }
    }

    /**
     * @dev get the deployer key
     * Set to default when it's in development environment
     * (uint for 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
     */
    function _helperGetDeployerInternalKey() private {
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77_814_517_325_470_205_911_140_941_194_401_928_579_557_062_014_761_831_930_645_393_041_380_819_009_408
            : vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        emit log_named_address("deployerAddress", deployerAddress);
        vm.startBroadcast(deployerPrivateKey);
    }

    /**
     * @dev when caller is owner of safe instance, prepare a signature and execute the transaction
     */
    function _helperSignSafeTxAsOwner(ISafe safe, address target, uint256 nonce, bytes memory data) private {
        bytes32 dataHash =
            safe.getTransactionHash(target, 0, data, Enum.Operation.Call, 0, 0, 0, address(0), msgSender, nonce);

        // sign dataHash
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(vm.envUint("PRIVATE_KEY"), dataHash);
        safe.execTransaction(
            target,
            0,
            data,
            Enum.Operation.Call,
            0,
            0,
            0,
            address(0),
            payable(address(msgSender)),
            abi.encodePacked(r, s, v)
        );
    }

    /**
     * @dev On network registry contract, register nodes and safes
     * This function should only be called by a manager
     */
    function registerNodes(address[] memory stakingAccounts, address[] memory nodeAddresses) public {
        // 1. get network and msg.sender
        getNetworkAndMsgSender();

        // 2. call private function that register nodes
        _registerNodes(stakingAccounts, nodeAddresses);

        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, register nodes and safes
     * This function should only be called by a manager
     */
    function _registerNodes(address[] memory stakingAccounts, address[] memory nodeAddresses) private {
        require(stakingAccounts.length == nodeAddresses.length, "Input lengths are different");

        // 1. check if nodes have been registered, if so, skip
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            (bool successReadRegisteredNodeAddress, bytes memory returndataRegisteredNodeAddress) = currentNetworkDetail
                .addresses
                .networkRegistryContractAddress
                .staticcall(abi.encodeWithSignature("nodeRegisterdToAccount(address)", nodeAddresses[i]));
            if (!successReadRegisteredNodeAddress) {
                revert("Cannot read successReadRegisteredNodeAddress from network registry contract.");
            }
            address registeredAccount = abi.decode(returndataRegisteredNodeAddress, (address));

            if (registeredAccount == address(0)) {
                accounts.push(stakingAccounts[i]);
                nodes.push(nodeAddresses[i]);
            }
        }

        // 2. register nodes
        if (nodes.length > 0) {
            (bool successRegisterNodes,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
                abi.encodeWithSignature("managerRegister(address[],address[])", accounts, nodes)
            );
            if (!successRegisterNodes) {
                emit log_string("Cannot register nodes as a manager");
                revert("Cannot register nodes as a manager");
            }
        }

        // reset
        accounts = new address[](0);
        nodes = new address[](0);
    }

    /**
     * @dev On network registry contract, deregister nodes from a set of addresses. This function should only be
     * called by a manager
     */
    function deregisterNodes(address[] calldata nodeAddresses) external {
        // 1. get network and msg.sender
        getNetworkAndMsgSender();

        // 2. check if nodes have been registered, if not, skip
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            (bool successReadRegisteredNodeAddress, bytes memory returndataRegisteredNodeAddress) = currentNetworkDetail
                .addresses
                .networkRegistryContractAddress
                .staticcall(abi.encodeWithSignature("nodeRegisterdToAccount(address)", nodeAddresses[i]));
            if (!successReadRegisteredNodeAddress) {
                revert("Cannot read successReadRegisteredNodeAddress from network registry contract.");
            }
            address registeredAccount = abi.decode(returndataRegisteredNodeAddress, (address));

            if (registeredAccount != address(0)) {
                nodes.push(nodeAddresses[i]);
            }
        }

        // 2. deregister nodes
        if (nodes.length > 0) {
            (bool successDeregisterNodes,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
                abi.encodeWithSignature("managerDeregister(address[])", nodes)
            );
            if (!successDeregisterNodes) {
                emit log_string("Cannot deregister nodes as a manager");
                revert("Cannot deregister nodes as a manager");
            }
        }

        // reset
        accounts = new address[](0);
        nodes = new address[](0);

        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, disable it. This function should only be called by the owner
     */
    function disableNetworkRegistry() external {
        // 1. get network and msg.sender
        getNetworkAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentNetworkDetail
            .addresses
            .networkRegistryContractAddress
            .staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. disable if needed
        if (isEnabled) {
            (bool successDisableNetworkRegistry,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
                abi.encodeWithSignature("disableRegistry()")
            );
            if (!successDisableNetworkRegistry) {
                emit log_string("Cannot disable network registery as a manager");
                revert("Cannotdisable network registery as a manager");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev On network registry contract, enable it. This function should only be called by a manager
     */
    function enableNetworkRegistry() external {
        // 1. get network and msg.sender
        getNetworkAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentNetworkDetail
            .addresses
            .networkRegistryContractAddress
            .staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. enable if needed
        if (!isEnabled) {
            (bool successEnableNetworkRegistry,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
                abi.encodeWithSignature("enableRegistry()")
            );
            if (!successEnableNetworkRegistry) {
                emit log_string("Cannot enable network registery as a manager");
                revert("Cannot enable network registery as a manager");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev On network registry contract, sync eligibility of some staking addresses. This function should only be
     * called by a manager
     */
    function syncEligibility(address[] calldata stakingAccounts) external {
        // 1. get network and msg.sender
        getNetworkAndMsgSender();

        // 2. sync peers eligibility according to the latest requirement of its current state
        (bool successSyncEligibility,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
            abi.encodeWithSignature("managerSync(address[])", stakingAccounts)
        );
        if (!successSyncEligibility) {
            emit log_string("Cannot sync eligibility as a manager");
            revert("Cannot sync eligibility as a manager");
        }
        vm.stopBroadcast();
    }

    // /**
    //  * @dev On stake contract, stake xHopr to the target value
    //  */
    // function stakeXHopr(uint256 stakeTarget) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. check the staked value. Return if the target has reached
    //   (bool successReadStaked, bytes memory returndataReadStaked) =
    // currentNetworkDetail.stakeContractAddress.staticcall(
    //     abi.encodeWithSignature('stakedHoprTokens(address)', msgSender)
    //   );
    //   if (!successReadStaked) {
    //     revert('Cannot read staked amount on stake contract.');
    //   }
    //   uint256 stakedAmount = abi.decode(returndataReadStaked, (uint256));
    //   if (stakedAmount >= stakeTarget) {
    //     emit log_string('Stake target has reached');
    //     return;
    //   }

    //   // 3. stake the difference, if allowed
    //   uint256 amountToStake = stakeTarget - stakedAmount;
    //   uint256 balance = _getTokenBalanceOf(currentNetworkDetail.xhoprTokenContractAddress, msgSender);
    //   if (stakedAmount >= stakeTarget) {
    //     emit log_string('Stake target has reached');
    //     return;
    //   }
    //   if (balance < amountToStake) {
    //     revert('Not enough xHOPR token balance to stake to the target.');
    //   } else {
    //     _stakeXHopr(currentNetworkDetail.xhoprTokenContractAddress, amountToStake);
    //   }
    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On stake contract, stake Network registry NFT to the target value
    //  */
    // function stakeNetworkRegistryNft(string calldata nftRank) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. Check if the msg.sender has staked Network_registry NFT
    //   if (checkHasStakedNetworkRegistryNft(currentNetworkDetail.stakeContractAddress, msgSender, nftRank)) return;

    //   // 3. Check if msg.sender has Network_registry NFT
    //   safeTransferNetworkRegistryNft(
    //     currentNetworkDetail.hoprBoostContractAddress,
    //     msgSender,
    //     currentNetworkDetail.stakeContractAddress,
    //     nftRank
    //   );

    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev Mint some xHOPR to the recipient
    //  */
    // function mintXHopr(address recipient, uint256 amountInEther) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   address[] memory addrBook = new address[](1);
    //   addrBook[0] = recipient;

    //   // 2. Check if the msg.sender has staked Network_registry NFT
    //   (bool successMintXTokens, ) = currentNetworkDetail.xhoprTokenContractAddress.call(
    //     abi.encodeWithSignature('batchMintInternal(address[],uint256)', addrBook, amountInEther * 1e18)
    //   );
    //   if (!successMintXTokens) {
    //     emit log_string('Cannot mint xHOPR tokens');
    //   }

    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev send some HOPR tokens to the recipient address
    //  */
    // function mintHopr(address recipient, uint256 tokenamountInEther) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2.Mint some Hopr tokens to the recipient
    //   if (tokenamountInEther > 0) {
    //     uint256 hoprTokenAmount = tokenamountInEther * 1 ether;
    //     (bool successMintTokens, ) = currentNetworkDetail.hoprTokenContractAddress.call(
    //       abi.encodeWithSignature('mint(address,uint256,bytes,bytes)', recipient, hoprTokenAmount, hex'00', hex'00')
    //     );
    //     if (!successMintTokens) {
    //       emit log_string('Cannot mint HOPR tokens');
    //     }
    //   }

    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev Check if msgSender owned the requested rank. If so, transfer one to recipient
    //  */
    // function transferNetworkRegistryNft(address recipient, string calldata nftRank) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. Check if msg.sender has Network_registry NFT
    //   safeTransferNetworkRegistryNft(currentNetworkDetail.hoprBoostContractAddress, msgSender, recipient, nftRank);
    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev Check if the address has staked Network_registry NFT
    //  */
    // function checkHasStakedNetworkRegistryNft(
    //   address stakeContractAddr,
    //   address stakingAccount,
    //   string calldata nftRank
    // ) private view returns (bool) {
    //   (bool successHasStaked, bytes memory returndataHasStaked) = stakeContractAddr.staticcall(
    //     abi.encodeWithSignature(
    //       'isNftTypeAndRankRedeemed2(uint256,string,address)',
    //       NETWORK_REGISTRY_NFT_INDEX,
    //       nftRank,
    //       stakingAccount
    //     )
    //   );
    //   if (!successHasStaked) {
    //     revert('Cannot read if the staking account has staked Network_registry NFTs.');
    //   }
    //   return abi.decode(returndataHasStaked, (bool));
    // }

    // /**
    //  * @dev Check if the address has staked Network_registry NFT
    //  */
    // function getMaxAllowedRegistrations(address proxyAddr, address stakingAccount) private view returns (uint256) {
    //   (bool successMaxAllowed, bytes memory returndataMaxAllowed) = proxyAddr.staticcall(
    //     abi.encodeWithSignature('maxAllowedRegistrations(address)', stakingAccount)
    //   );
    //   if (!successMaxAllowed) {
    //     revert('Cannot read maxAllowedRegistrations for staking account.');
    //   }
    //   return abi.decode(returndataMaxAllowed, (uint256));
    // }

    // /**
    //  * @dev private function to transfer a NR NFT of nftRank from sender to recipient.
    //  */
    // function safeTransferNetworkRegistryNft(
    //   address boostContractAddr,
    //   address sender,
    //   address recipient,
    //   string calldata nftRank
    // ) private {
    //   // check if the sender owns the desired nft rank
    //   (bool ownsNft, uint256 tokenId) = _hasNetworkRegistryNft(boostContractAddr, sender, nftRank);

    //   if (!ownsNft) {
    //     revert('Failed to find the owned NFT');
    //   }

    //   // found the tokenId, perform safeTransferFrom
    //   _stakeNft(boostContractAddr, sender, recipient, tokenId);
    // }

    /**
     * @dev This function funds a recipient wallet with HOPR tokens and native tokens, but only when the recipient has
     * not yet received
     * enough value.
     * First, HOPR tokens are prioritized to be transferred than minted to the recipient
     * Native tokens are transferred to the recipient
     * @param recipient The address of the recipient wallet.
     * @param hoprTokenAmountInWei, The amount of HOPR tokens that recipient is desired to receive
     * @param nativeTokenAmountInWei The amount of native tokens that recipient is desired to receive
     */
    function transferOrMintHoprAndSendNativeToAmount(
        address recipient,
        uint256 hoprTokenAmountInWei,
        uint256 nativeTokenAmountInWei
    )
        public
        payable
    {
        // 1. get environment and msg.sender
        getNetworkAndMsgSender();

        // 2. transfer or mint hopr tokens
        _transferOrMintHoprToAmount(
            currentNetworkDetail.addresses.tokenContractAddress, recipient, hoprTokenAmountInWei
        );

        // 3. transfer native balance to the recipient
        if (nativeTokenAmountInWei > recipient.balance) {
            (bool nativeTokenTransferSuccess,) = recipient.call{ value: nativeTokenAmountInWei - recipient.balance }("");
            require(nativeTokenTransferSuccess, "Cannot send native tokens to the recipient");
        }
        vm.stopBroadcast();
    }

    // /**
    //  * @dev private function to check if an account owns a Network Registry NFT of nftRank
    //  */
    // function _hasNetworkRegistryNft(
    //   address boostContractAddr,
    //   address account,
    //   string memory nftRank
    // ) private view returns (bool ownsNft, uint256 tokenId) {
    //   // 1. Check account's Network_registry NFT balance
    //   uint256 ownedNftBalance = _getTokenBalanceOf(boostContractAddr, account);
    //   // get the desired nft uri hash
    //   string memory desiredTokenUriPart = string(abi.encodePacked(NETWORK_REGISTRY_TYPE_NAME, '/', nftRank));

    //   // 2. Loop through balance and compare token URI
    //   uint256 index;
    //   for (index = 0; index < ownedNftBalance; index++) {
    //     (bool successOwnedNftTokenId, bytes memory returndataOwnedNftTokenId) = boostContractAddr.staticcall(
    //       abi.encodeWithSignature('tokenOfOwnerByIndex(address,uint256)', account, index)
    //     );
    //     if (!successOwnedNftTokenId) {
    //       revert('Cannot read owned NFT at a given index.');
    //     }
    //     uint256 ownedNftTokenId = abi.decode(returndataOwnedNftTokenId, (uint256));
    //     (bool successTokenUri, bytes memory returndataTokenUri) = boostContractAddr.staticcall(
    //       abi.encodeWithSignature('tokenURI(uint256)', ownedNftTokenId)
    //     );
    //     if (!successTokenUri) {
    //       revert('Cannot read token URI of the given ID.');
    //     }
    //     string memory tokenUri = abi.decode(returndataTokenUri, (string));

    //     if (_hasSubstring(tokenUri, desiredTokenUriPart)) {
    //       // 3. find the tokenId
    //       ownsNft = true;
    //       tokenId = ownedNftTokenId;
    //       break;
    //     }
    //   }
    //   return (ownsNft, tokenId);
    // }

    /**
     * Get the token balance of a wallet
     */
    function _getTokenBalanceOf(address tokenAddress, address wallet) internal view returns (uint256) {
        (bool successReadOwnedTokens, bytes memory returndataReadOwnedTokens) =
            tokenAddress.staticcall(abi.encodeWithSignature("balanceOf(address)", wallet));
        if (!successReadOwnedTokens) {
            revert FailureInReadBalance(tokenAddress);
        }
        return abi.decode(returndataReadOwnedTokens, (uint256));
    }

    // /**
    //  * ported from HoprStakeBase.sol
    //  * @dev if the given `tokenURI` end with `/substring`
    //  * @param tokenURI string URI of the HoprBoost NFT. E.g. "https://stake.hoprnet.org/PuzzleHunt_v2/Bronze - Week
    // 5"
    //  * @param substring string of the `boostRank` or `boostType/boostRank`. E.g. "Bronze - Week 5",
    // "PuzzleHunt_v2/Bronze - Week 5"
    //  */
    // function _hasSubstring(string memory tokenURI, string memory substring) internal pure returns (bool) {
    //   // convert string to bytes
    //   bytes memory tokenURIInBytes = bytes(tokenURI);
    //   bytes memory substringInBytes = bytes(substring);

    //   // length of tokenURI is the sum of substringLen and restLen, where
    //   // - `substringLen` is the length of the part that is extracted and compared with the provided substring
    //   // - `restLen` is the length of the baseURI and boostType, which will be offset
    //   uint256 substringLen = substringInBytes.length;
    //   uint256 restLen = tokenURIInBytes.length - substringLen;
    //   // one byte before the supposed substring, to see if it's the start of `substring`
    //   bytes1 slashPositionContent = tokenURIInBytes[restLen - 1];

    //   if (slashPositionContent != 0x2f) {
    //     // if this position is not a `/`, substring in the tokenURI is for sure neither `boostRank` nor
    // `boostType/boostRank`
    //     return false;
    //   }

    //   // offset so that value from the next calldata (`substring`) is removed, so bitwise it needs to shift
    //   // log2(16) * (32 - substringLen) * 2
    //   uint256 offset = (32 - substringLen) * 8;

    //   bytes32 trimed; // left-padded extracted `boostRank` from the `tokenURI`
    //   bytes32 substringInBytes32 = bytes32(substringInBytes); // convert substring in to bytes32
    //   bytes32 shifted; // shift the substringInBytes32 from right-padded to left-padded

    //   bool result;
    //   assembly {
    //     // assuming `boostRank` or `boostType/boostRank` will never exceed 32 bytes
    //     // left-pad the `boostRank` extracted from the `tokenURI`, so that possible
    //     // extra pieces of `substring` is not included
    //     // 32 jumps the storage of bytes length and restLen offsets the `baseURI`
    //     trimed := shr(offset, mload(add(add(tokenURIInBytes, 32), restLen)))
    //     // tokenURIInBytes32 := mload(add(add(tokenURIInBytes, 32), restLen))
    //     // left-pad `substring`
    //     shifted := shr(offset, substringInBytes32)
    //     // compare results
    //     result := eq(trimed, shifted)
    //   }
    //   return result;
    // }

    // function _stakeXHopr(address xhoprTokenContract, uint256 amountToStake) private {
    //   (bool successStakeXhopr, ) = currentNetworkDetail.xhoprTokenContractAddress.call(
    //     abi.encodeWithSignature(
    //       'transferAndCall(address,uint256,bytes)',
    //       currentNetworkDetail.stakeContractAddress,
    //       amountToStake,
    //       hex'00'
    //     )
    //   );
    //   if (!successStakeXhopr) {
    //     emit log_string('Cannot stake amountToStake');
    //     revert('Cannot stake amountToStake');
    //   }
    // }

    // function _stakeNft(address boostContractAddr, address sender, address recipient, uint256 tokenId) private {
    //   (bool successStakeNft, ) = boostContractAddr.call(
    //     abi.encodeWithSignature('safeTransferFrom(address,address,uint256)', sender, recipient, tokenId)
    //   );
    //   if (!successStakeNft) {
    //     revert('Cannot stake the NFT');
    //   }
    // }

    // function _selfRegisterNodes(address networkRegistryContractAddress, string[] calldata peerIds) private {
    //   // 2. call hoprNetworkRegistry.selfRegister(peerIds);
    //   (bool successSelfRegister, ) = networkRegistryContractAddress.call(
    //     abi.encodeWithSignature('selfRegister(string[])', peerIds)
    //   );
    //   if (!successSelfRegister) {
    //     emit log_string('Cannot register peers');
    //     revert('Cannot register peers');
    //   }
    // }

    function _transferOrMintHoprToAmount(
        address hoprTokenContractAddress,
        address recipient,
        uint256 hoprTokenAmountInWei
    )
        private
    {
        // 1. get recipient balance
        uint256 recipientTokenBalance = _getTokenBalanceOf(hoprTokenContractAddress, recipient);

        // 2. transfer some Hopr tokens to the recipient, or mint tokens
        if (hoprTokenAmountInWei > recipientTokenBalance) {
            // get the difference to transfer
            uint256 hoprTokenToTransfer = hoprTokenAmountInWei - recipientTokenBalance;
            // check the hopr token balance
            uint256 senderHoprTokenBalance = _getTokenBalanceOf(hoprTokenContractAddress, msgSender);

            if (senderHoprTokenBalance >= hoprTokenToTransfer) {
                // call transfer
                (bool successTransfserTokens,) = hoprTokenContractAddress.call(
                    abi.encodeWithSignature("transfer(address,uint256)", recipient, hoprTokenToTransfer)
                );
                if (!successTransfserTokens) {
                    emit log_string("Cannot transfer HOPR tokens to the recipient");
                }
            } else {
                // if transfer cannot be called, try minting token as a minter
                bytes32 MINTER_ROLE = keccak256("MINTER_ROLE");
                (bool successHasRole, bytes memory returndataHasRole) = hoprTokenContractAddress.staticcall(
                    abi.encodeWithSignature("hasRole(bytes32,address)", MINTER_ROLE, msgSender)
                );
                if (!successHasRole) {
                    revert("Cannot check role for Hopr token.");
                }
                bool isMinter = abi.decode(returndataHasRole, (bool));
                require(isMinter, "Caller is not a minter");

                (bool successMintTokens,) = hoprTokenContractAddress.call(
                    abi.encodeWithSignature(
                        "mint(address,uint256,bytes,bytes)", recipient, hoprTokenToTransfer, hex"00", hex"00"
                    )
                );
                if (!successMintTokens) {
                    emit log_string("Cannot mint HOPR tokens to the recipient");
                }
            }
        }
    }
}
