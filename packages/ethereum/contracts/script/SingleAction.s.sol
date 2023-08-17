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
    ) public view virtual returns (bytes32);

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
    ) public payable virtual returns (bool success);

    function nonce() public virtual returns (uint256);
}

abstract contract IFactory {
    function clone(address moduleSingletonAddress, address[] memory admins, uint256 nonce, bytes32 defaultTarget)
        public
        virtual
        returns (address, address payable);
}

/// Failed to read balance of a token contract
/// @param token token address.
error FailureInReadBalance(address token);

/**
 * @dev script to interact with contract(s) of a given environment where the msg.sender comes from the environment variable `PRIVATE_KEY`
 * Private key of the caller must be saved under the environment variable `PRIVATE_KEY`
 * Wrapper of contracts (incl. NetworkRegistery, HoprStake) with detection of contract address per network/environment_type
 */
contract SingleActionFromPrivateKeyScript is Test, NetworkConfig {
    using stdJson for string;
    using BoostUtilsLib for address;
    using TargetUtils for Target;

    address msgSender;
    string[] private unregisteredIds;

    function getNetworkAndMsgSender() private {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();

        // 2. Get private key of caller
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        msgSender = vm.addr(privateKey);
        vm.startBroadcast(privateKey);
    }

    /**
     * @dev create a safe proxy and moodule proxy
     * @notice Deployer is the single owner of safe
     * nonce is the current nonce of deployer account
     * Default fallback permission for module is to allow all except for node sending xDAI from safe
     * Include a node to the safe module
     * @param nodeAddresses array of node addresses to be added to the module
     */
    function expressSetupSafeModule(address[] memory nodeAddresses) external returns (address safe, address module) {
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
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultApproveFunctionPermisson
         *       CapabilityPermission.SPECIFIC_FALLBACK_BLOCK  // defaultSendFunctionPermisson
         *     ]
         */
        CapabilityPermission[] memory defaultChannelsCapabilityPermissions = new CapabilityPermission[](9);
        for (uint256 i = 0; i < defaultChannelsCapabilityPermissions.length; i++) {
            defaultChannelsCapabilityPermissions[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        defaultChannelsCapabilityPermissions[8] = CapabilityPermission.SPECIFIC_FALLBACK_BLOCK;
        Target defaultModulePermission = TargetUtils.encodeDefaultPermissions(
            currentNetworkDetail.addresses.channelsContractAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.SPECIFIC_FALLBACK_BLOCK,
            defaultChannelsCapabilityPermissions
        );

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
        Target[] memory defaultNodeTargets = new Target[](nodeAddresses.length);
        for (uint256 j = 0; j < nodeAddresses.length; j++) {
            defaultNodeTargets[j] = TargetUtils.encodeDefaultPermissions(
                nodeAddresses[j],
                Clearance.FUNCTION,
                TargetType.SEND,
                TargetPermission.SPECIFIC_FALLBACK_BLOCK,
                nodeDefaultPermission
            );
        }

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

        // 4. include node to the module, as an owner of safe
        for (uint256 k = 0; k < nodeAddresses.length; k++) {
            bytes memory includeNodeData =
                abi.encodeWithSignature("includeNode(uint256)", Target.unwrap(defaultNodeTargets[k]));
            uint256 safeNonce = ISafe(safe).nonce();
            _helperSignSafeTxAsOwner(ISafe(safe), module, safeNonce, includeNodeData);
        }
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

    // TODO: reimplement single actions
    // /**
    //  * @dev express node initialization
    //  * - Check with network registery on the registeration status of a list of peer ids, return the unregistered ones.
    //  * - If not all the peer ids are registered, check if the caller can do selfRegister
    //  */
    // function expressInitialization(
    //   address[] calldata nodeAddrs,
    //   uint256 hoprTokenAmountInWei,
    //   uint256 nativeTokenAmountInWei,
    //   string[] calldata peerIds
    // ) external {
    //   // 1. get environment and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. loop through nodes and check its registration status
    //   for (uint256 index = 0; index < peerIds.length; index++) {
    //     (bool successCheck, bytes memory returndataCheck) = currentNetworkDetail
    //       .networkRegistryContractAddress
    //       .staticcall(abi.encodeWithSignature('nodePeerIdToAccount(string)', peerIds[index]));
    //     if (!successCheck) {
    //       revert('Cannot read nodePeerIdToAccount from network registry contract.');
    //     }
    //     address stakingAccount = abi.decode(returndataCheck, (address));
    //     if (stakingAccount == address(0)) {
    //       unregisteredIds.push(peerIds[index]);
    //     }
    //   }
    //   uint256 numUnRegisteredIds = unregisteredIds.length;

    //   // 3. check if need to perform node registeration.
    //   // As NR is disabled in development, skip it
    //   if (numUnRegisteredIds > 0 && currentEnvironmentType != EnvironmentType.DEVELOPMENT) {
    //     // check if the caller can register nodes
    //     uint256 allowedRegistration = getMaxAllowedRegistrations(
    //       currentNetworkDetail.networkRegistryProxyContractAddress,
    //       msgSender
    //     );
    //     if (allowedRegistration < numUnRegisteredIds) {
    //       // try to register developer NFT, community NFT or stake HOPR tokens
    //       // check if the caller owns developer NFT
    //       uint256 nftTokenId;
    //       (bool ownsDevNft, uint256 devTokenId) = _hasNetworkRegistryNft(
    //         currentNetworkDetail.hoprBoostContractAddress,
    //         msgSender,
    //         NETWORK_REGISTRY_RANK1_NAME
    //       );
    //       (bool ownsComNft, uint256 comTokenId) = _hasNetworkRegistryNft(
    //         currentNetworkDetail.hoprBoostContractAddress,
    //         msgSender,
    //         NETWORK_REGISTRY_RANK2_NAME
    //       );
    //       uint256 hoprBalance = _getTokenBalanceOf(currentNetworkDetail.hoprTokenContractAddress, msgSender);

    //       if (!ownsDevNft && !ownsComNft) {
    //         // try to stake HOPR tokens
    //         _stakeXHopr(currentNetworkDetail.xhoprTokenContractAddress, 1000 ether * numUnRegisteredIds);
    //       } else {
    //         // try to stake NFT
    //         nftTokenId = ownsDevNft ? devTokenId : comTokenId;
    //         _stakeNft(
    //           currentNetworkDetail.hoprBoostContractAddress,
    //           msgSender,
    //           currentNetworkDetail.stakeContractAddress,
    //           nftTokenId
    //         );
    //       }
    //     }
    //     // try again registration
    //     _selfRegisterNodes(currentNetworkDetail.networkRegistryContractAddress, peerIds);
    //   }

    //   // 4. loop again and check if need to fund nodes
    //   for (uint256 nodeIndex = 0; nodeIndex < nodeAddrs.length; nodeIndex++) {
    //     address recipient = nodeAddrs[nodeIndex];
    //     // transfer or mint hopr tokens
    //     _transferOrMintHoprToAmount(currentNetworkDetail.hoprTokenContractAddress, recipient, hoprTokenAmountInWei);

    //     // 3. transfer native balance to the unregisteredIds[numUnRegisteredIndex]
    //     if (nativeTokenAmountInWei > recipient.balance) {
    //       (bool nativeTokenTransferSuccess, ) = recipient.call{value: nativeTokenAmountInWei - recipient.balance}('');
    //       require(nativeTokenTransferSuccess, 'Cannot send native tokens to the recipient');
    //     }
    //   }

    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On network registry contract, register peers associated with the calling wallet.
    //  */
    // function selfRegisterNodes(string[] calldata peerIds) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. call hoprNetworkRegistry.selfRegister(peerIds);
    //   _selfRegisterNodes(currentNetworkDetail.networkRegistryContractAddress, peerIds);

    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On network registry contract, deregister peers associated with the calling wallet.
    //  */
    // function selfDeregisterNodes(string[] calldata peerIds) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. call hoprNetworkRegistry.selfDeregister(peerIds);
    //   (bool successSelfDeregister, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //     abi.encodeWithSignature('selfDeregister(string[])', peerIds)
    //   );
    //   if (!successSelfDeregister) {
    //     emit log_string('Cannot deregister peers');
    //     revert('Cannot deregister peers');
    //   }
    //   vm.stopBroadcast();
    // }

    /**
     * @dev On network registry contract, register nodes and safes
     * This function should only be called by a manager
     */
    function registerNodes(address[] calldata stakingAccounts, address[] calldata nodeAddresses) external {
      require(stakingAccounts.length == nodeAddresses.length, 'Input lengths are different');

      // 1. get network and msg.sender
      getNetworkAndMsgSender();

      // 2. register nodes
      (bool successRegisterNodes, ) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
        abi.encodeWithSignature('managerRegister(address[],address[])', stakingAccounts, nodeAddresses)
      );
      if (!successRegisterNodes) {
        emit log_string('Cannot register nodes as a manager');
        revert('Cannot register nodes as a manager');
      }
      vm.stopBroadcast();
    }

    // /**
    //  * @dev On network registry contract, deregister nodes from a set of addresses. This function should only be called by the owner
    //  */
    // function deregisterNodes(address[] calldata stakingAddresses, string[] calldata peerIds) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. owner registers nodes, depending on the environment
    //   if (currentEnvironmentType == EnvironmentType.LOCAL) {
    //     // call deregister accounts on HoprDummyProxyForNetworkRegistry
    //     (bool successDeregisterNodesOnDummyProxy, ) = currentNetworkDetail.networkRegistryProxyContractAddress.call(
    //       abi.encodeWithSignature('ownerBatchRemoveAccounts(address[])', stakingAddresses)
    //     );
    //     if (!successDeregisterNodesOnDummyProxy) {
    //       emit log_string('Cannot remove stakingAddresses from the dummy proxy.');
    //       revert('Cannot remove stakingAddresses from the dummy proxy.');
    //     }
    //   }
    //   // actual deregister nodes
    //   (bool successDeregisterNodes, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //     abi.encodeWithSignature('ownerDeregister(string[])', peerIds)
    //   );
    //   if (!successDeregisterNodes) {
    //     emit log_string('Cannot rdeegister nodes as an owner');
    //     revert('Cannot deregister nodes as an owner');
    //   }
    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On network registry contract, disable it. This function should only be called by the owner
    //  */
    // function disableNetworkRegistry() external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. check if current NR is enabled.
    //   (bool successReadEnabled, bytes memory returndataReadEnabled) = currentNetworkDetail
    //     .networkRegistryContractAddress
    //     .staticcall(abi.encodeWithSignature('enabled()'));
    //   if (!successReadEnabled) {
    //     revert('Cannot read enabled from network registry contract.');
    //   }
    //   bool isEnabled = abi.decode(returndataReadEnabled, (bool));

    //   // 3. disable if needed
    //   if (isEnabled) {
    //     (bool successDisableNetworkRegistry, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //       abi.encodeWithSignature('disableRegistry()')
    //     );
    //     if (!successDisableNetworkRegistry) {
    //       emit log_string('Cannot disable network registery as an owner');
    //       revert('Cannotdisable network registery as an owner');
    //     }
    //     vm.stopBroadcast();
    //   }
    // }

    // /**
    //  * @dev On network registry contract, enable it. This function should only be called by the owner
    //  */
    // function enableNetworkRegistry() external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. check if current NR is enabled.
    //   (bool successReadEnabled, bytes memory returndataReadEnabled) = currentNetworkDetail
    //     .networkRegistryContractAddress
    //     .staticcall(abi.encodeWithSignature('enabled()'));
    //   if (!successReadEnabled) {
    //     revert('Cannot read enabled from network registry contract.');
    //   }
    //   bool isEnabled = abi.decode(returndataReadEnabled, (bool));

    //   // 3. enable if needed
    //   if (!isEnabled) {
    //     (bool successEnableNetworkRegistry, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //       abi.encodeWithSignature('enableRegistry()')
    //     );
    //     if (!successEnableNetworkRegistry) {
    //       emit log_string('Cannot enable network registery as an owner');
    //       revert('Cannot enable network registery as an owner');
    //     }
    //     vm.stopBroadcast();
    //   }
    // }

    // /**
    //  * @dev On network registry contract, update eligibility of some staking addresses to the desired . This function should only be called by the owner
    //  */
    // function forceEligibilityUpdate(address[] calldata stakingAddresses, bool[] calldata eligibility) external {
    //   require(stakingAddresses.length == eligibility.length, 'Input lengths are different');

    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. update emit EligibilityUpdate events by the owner
    //   (bool successForceEligibilityUpdate, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //     abi.encodeWithSignature('ownerForceEligibility(address[],bool[])', stakingAddresses, eligibility)
    //   );
    //   if (!successForceEligibilityUpdate) {
    //     emit log_string('Cannot force update eligibility as an owner');
    //     revert('Cannot force update eligibility as an owner');
    //   }
    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On network registry contract, sync eligibility of some staking addresses. This function should only be called by the owner
    //  */
    // function syncEligibility(string[] calldata peerIds) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. sync peers eligibility according to the latest requirement of its current state
    //   (bool successSyncEligibility, ) = currentNetworkDetail.networkRegistryContractAddress.call(
    //     abi.encodeWithSignature('sync(string[])', peerIds)
    //   );
    //   if (!successSyncEligibility) {
    //     emit log_string('Cannot sync eligibility as an owner');
    //     revert('Cannot sync eligibility as an owner');
    //   }
    //   vm.stopBroadcast();
    // }

    // /**
    //  * @dev On stake contract, stake xHopr to the target value
    //  */
    // function stakeXHopr(uint256 stakeTarget) external {
    //   // 1. get network and msg.sender
    //   getNetworkAndMsgSender();

    //   // 2. check the staked value. Return if the target has reached
    //   (bool successReadStaked, bytes memory returndataReadStaked) = currentNetworkDetail.stakeContractAddress.staticcall(
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
     * @dev This function funds a recipient wallet with HOPR tokens and native tokens, but only when the recipient has not yet received
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
    ) external payable {
        // 1. get environment and msg.sender
        getNetworkAndMsgSender();

        // 2. transfer or mint hopr tokens
        _transferOrMintHoprToAmount(
            currentNetworkDetail.addresses.tokenContractAddress, recipient, hoprTokenAmountInWei
        );

        // 3. transfer native balance to the recipient
        if (nativeTokenAmountInWei > recipient.balance) {
            (bool nativeTokenTransferSuccess,) = recipient.call{value: nativeTokenAmountInWei - recipient.balance}("");
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
    //  * @param tokenURI string URI of the HoprBoost NFT. E.g. "https://stake.hoprnet.org/PuzzleHunt_v2/Bronze - Week 5"
    //  * @param substring string of the `boostRank` or `boostType/boostRank`. E.g. "Bronze - Week 5", "PuzzleHunt_v2/Bronze - Week 5"
    //  */
    // function _hasSubstring(string memory tokenURI, string memory substring) internal pure returns (bool) {
    //   // convert string to bytes
    //   bytes memory tokenURIInBytes = bytes(tokenURI);
    //   bytes memory substringInBytes = bytes(substring);

    //   // lenghth of tokenURI is the sum of substringLen and restLen, where
    //   // - `substringLen` is the length of the part that is extracted and compared with the provided substring
    //   // - `restLen` is the length of the baseURI and boostType, which will be offset
    //   uint256 substringLen = substringInBytes.length;
    //   uint256 restLen = tokenURIInBytes.length - substringLen;
    //   // one byte before the supposed substring, to see if it's the start of `substring`
    //   bytes1 slashPositionContent = tokenURIInBytes[restLen - 1];

    //   if (slashPositionContent != 0x2f) {
    //     // if this position is not a `/`, substring in the tokenURI is for sure neither `boostRank` nor `boostType/boostRank`
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
    ) private {
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
