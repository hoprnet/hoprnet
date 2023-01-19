// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/Test.sol";
import "./utils/EnvironmentConfig.s.sol";
import "./utils/BoostUtilsLib.sol";

/**
 * @dev script to interact with contract(s) of a given envirionment where the msg.sender comes from the environment variable `PRIVATE_KEY`
 * Private key of the caller must be saved under the envrionment variable `PRIVATE_KEY`
 * Wrapper of contracts (incl. NetworkRegistery, HoprStake) with detection of contract address per environment_name/environment_type
 */
contract SingleActionFromPrivateKeyScript is Test, EnvironmentConfig {
    using stdJson for string;
    using BoostUtilsLib for address;

    address msgSender;

    function getEnvironmentAndMsgSender() private {
        // 1. Environment check
        // get envirionment of the script
        getEnvironment();
        // read records of deployed files
        readCurrentEnvironment();

        // 2. Get private key of caller
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        msgSender = vm.addr(privateKey);
        vm.startBroadcast(privateKey);
    }

    /**
     * @dev On network registry contract, register peers associated with the calling wallet.
     */
    function selfRegisterNodes(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. call hoprNetworkRegistry.selfRegister(peerIds);
        (bool successSelfRegister, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("selfRegister(string[])", peerIds));
        if (!successSelfRegister) {
            emit log_string("Cannot register peers");
            revert("Cannot register peers");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, deregister peers associated with the calling wallet.
     */
    function selfDeregisterNodes(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. call hoprNetworkRegistry.selfDeregister(peerIds);
        (bool successSelfDeregister, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("selfDeregister(string[])", peerIds));
        if (!successSelfDeregister) {
            emit log_string("Cannot deregister peers");
            revert("Cannot deregister peers");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, register nodes to a set of addresses. This function should only be called by the owner
     */
    function registerNodes(address[] calldata stakingAddresses, string[] calldata peerIds) external {
        require(stakingAddresses.length == peerIds.length, "Input lengths are different");

        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. owner registers nodes, depending on the envirionment
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
            // call register accounts on HoprDummyProxyForNetworkRegistry
            (bool successRegisterNodesOnDummyProxy, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("ownerBatchAddAccounts(address[])", stakingAddresses));
            if (!successRegisterNodesOnDummyProxy) {
                emit log_string("Cannot add stakingAddresses on to the dummy proxy.");
                revert("Cannot add stakingAddresses on to the dummy proxy.");
            }
        }
        // actual register nodes
        (bool successRegisterNodes, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerRegister(address[],string[])", stakingAddresses, peerIds));
        if (!successRegisterNodes) {
            emit log_string("Cannot register nodes as an owner");
            revert("Cannot register nodes as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, deregister nodes from a set of addresses. This function should only be called by the owner
     */
    function deregisterNodes(address[] calldata stakingAddresses, string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. owner registers nodes, depending on the envirionment
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
            // call deregister accounts on HoprDummyProxyForNetworkRegistry
            (bool successDeregisterNodesOnDummyProxy, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("ownerBatchRemoveAccounts(address[])", stakingAddresses));
            if (!successDeregisterNodesOnDummyProxy) {
                emit log_string("Cannot remove stakingAddresses from the dummy proxy.");
                revert("Cannot remove stakingAddresses from the dummy proxy.");
            }
        }
        // actual deregister nodes
        (bool successDeregisterNodes, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerDeregister(string[])", peerIds));
        if (!successDeregisterNodes) {
            emit log_string("Cannot rdeegister nodes as an owner");
            revert("Cannot deregister nodes as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, disable it. This function should only be called by the owner
     */
    function disableNetworkRegistry() external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. disable if needed
        if (isEnabled) {
            (bool successDisableNetworkRegistry, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("disableRegistry()"));
            if (!successDisableNetworkRegistry) {
                emit log_string("Cannot disable network registery as an owner");
                revert("Cannotdisable network registery as an owner");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev On network registry contract, enable it. This function should only be called by the owner
     */
    function enableNetworkRegistry() external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. check if current NR is enabled.
        (bool successReadEnabled, bytes memory returndataReadEnabled) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("enabled()"));
        if (!successReadEnabled) {
            revert("Cannot read enabled from network registry contract.");
        }
        bool isEnabled = abi.decode(returndataReadEnabled, (bool));

        // 3. enable if needed
        if (!isEnabled) {
            (bool successEnableNetworkRegistry, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("enableRegistry()"));
            if (!successEnableNetworkRegistry) {
                emit log_string("Cannot enable network registery as an owner");
                revert("Cannot enable network registery as an owner");
            }
            vm.stopBroadcast();
        }
    }

    /**
     * @dev On network registry contract, update eligibility of some staking addresses to the desired . This function should only be called by the owner
     */
    function forceEligibilityUpdate(address[] calldata stakingAddresses, bool[] calldata eligibility) external {
        require(stakingAddresses.length == eligibility.length, "Input lengths are different");

        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. update emit EligibilityUpdate events by the owner
        (bool successForceEligibilityUpdate, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("ownerForceEligibility(address[],bool[])", stakingAddresses, eligibility));
        if (!successForceEligibilityUpdate) {
            emit log_string("Cannot force update eligibility as an owner");
            revert("Cannot force update eligibility as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On network registry contract, sync eligibility of some staking addresses. This function should only be called by the owner
     */
    function syncEligibility(string[] calldata peerIds) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. sync peers eligibility according to the latest requirement of its current state
        (bool successSyncEligibility, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("sync(string[])", peerIds));
        if (!successSyncEligibility) {
            emit log_string("Cannot sync eligibility as an owner");
            revert("Cannot sync eligibility as an owner");
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On stake contract, stake xHopr to the target value
     */
    function stakeXHopr(uint256 stakeTarget) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. check the staked value. Return if the target has reached
        (bool successReadStaked, bytes memory returndataReadStaked) = currentEnvironmentDetail.stakeContractAddress.staticcall(abi.encodeWithSignature("stakedHoprTokens(address)", msgSender));
        if (!successReadStaked) {
            revert("Cannot read staked amount on stake contract.");
        }
        uint256 stakedAmount = abi.decode(returndataReadStaked, (uint256));
        if (stakedAmount >= stakeTarget) {
            emit log_string("Stake target has reached");
            return;
        }

        // 3. stake the difference, if allowed
        uint256 amountToStake = stakeTarget - stakedAmount;
        (bool successReadBalance, bytes memory returndataReadBalance) = currentEnvironmentDetail.xhoprTokenContractAddress.staticcall(abi.encodeWithSignature("balanceOf(address)", msgSender));
        if (!successReadBalance) {
            revert("Cannot read token balance on xHOPR token contract.");
        }
        uint256 balance = abi.decode(returndataReadBalance, (uint256));
        if (stakedAmount >= stakeTarget) {
            emit log_string("Stake target has reached");
            return;
        }
        if (balance < amountToStake) {
            revert("Not enough xHOPR token balance to stake to the target.");
        } else {
            (bool successStakeXhopr, ) = currentEnvironmentDetail.xhoprTokenContractAddress.call(abi.encodeWithSignature("transferAndCall(address,uint256,bytes)", currentEnvironmentDetail.stakeContractAddress, amountToStake, hex"00"));
            if (!successStakeXhopr) {
                emit log_string("Cannot stake amountToStake");
                revert("Cannot stake amountToStake");
            }
        }
        vm.stopBroadcast();
    }

    /**
     * @dev On stake contract, stake Network registry NFT to the target value
     */
    function stakeNetworkRegistryNft(string calldata nftRank) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. Check if the msg.sender has staked Network_registry NFT
        (bool successHasStaked, bytes memory returndataHasStaked) = currentEnvironmentDetail.stakeContractAddress.staticcall(abi.encodeWithSignature("isNftTypeAndRankRedeemed2(uint256,string,address)", NETWORK_REGISTRY_NFT_INDEX, nftRank, msgSender));
        if (!successHasStaked) {
            revert("Cannot read if caller has staked Network_registry NFTs.");
        }
        bool hasStaked = abi.decode(returndataHasStaked, (bool));
        if (hasStaked) {
            return;
        }

        // 3. Check if msg.sender has Network_registry NFT
        safeTransferNetworkRegistryNft(currentEnvironmentDetail.hoprBoostContractAddress, msgSender, currentEnvironmentDetail.stakeContractAddress, nftRank);

        vm.stopBroadcast();
    }

    /**
     * @dev Mint some xHOPR to the recipient
     */
    function mintXHopr(address recipient, uint256 amountInEther) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        address[] memory addrBook = new address[](1);
        addrBook[0] = recipient;

        // 2. Check if the msg.sender has staked Network_registry NFT
        (bool successMintXTokens, ) = currentEnvironmentDetail.xhoprTokenContractAddress.call(abi.encodeWithSignature("batchMintInternal(address[],uint256)", addrBook, amountInEther * 1e18));
        if (!successMintXTokens) {
            emit log_string("Cannot mint xHOPR tokens");
        }

        vm.stopBroadcast();
    }

    /**
     * @dev send some HOPR tokens to the recipient address
     */
    function mintHopr(address recipient, uint256 tokenamountInEther) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2.Mint some Hopr tokens to the recipient
        if (tokenamountInEther > 0) {
            uint256 hoprTokenAmount = tokenamountInEther * 1 ether;
            (bool successMintTokens, ) = currentEnvironmentDetail.hoprTokenContractAddress.call(abi.encodeWithSignature("mint(address,uint256,bytes,bytes)", recipient, hoprTokenAmount, hex"00", hex"00"));
            if (!successMintTokens) {
                emit log_string("Cannot mint HOPR tokens");
            }
        }

        vm.stopBroadcast();
    }

    /**
     * @dev Check if msgSender owned the requested rank. If so, transfer one to recipient
     */
    function transferNetworkRegistryNft(address recipient, string calldata nftRank) external {
        // 1. get environment and msg.sender
        getEnvironmentAndMsgSender();

        // 2. Check if msg.sender has Network_registry NFT
        safeTransferNetworkRegistryNft(currentEnvironmentDetail.hoprBoostContractAddress, msgSender, recipient, nftRank);
        vm.stopBroadcast();
    }

    /**
     * @dev private function to transfer a NR NFT of nftRank from sender to recipient. 
     */
    function safeTransferNetworkRegistryNft(address boostContractAddr, address sender, address recipient, string calldata nftRank) private {
        // 1. Check sender's Network_registry NFT balance
        (bool successOwnedNftBalance, bytes memory returndataOwnedNftBalance) = boostContractAddr.staticcall(abi.encodeWithSignature("balanceOf(address)", sender));
        if (!successOwnedNftBalance) {
            revert("Cannot read if the amount of Network_registry NFTs owned by the caller.");
        }
        uint256 ownedNftBalance = abi.decode(returndataOwnedNftBalance, (uint256));
        // get the desired nft uri hash
        bytes32 desiredHaashedTokenUri = keccak256(bytes(abi.encodePacked(NETWORK_REGISTRY_TYPE_NAME, "/", nftRank)));

        // 2. Loop through balance and compare token URI
        uint256 index;
        for (index = 0; index < ownedNftBalance; index++) {
            (bool successOwnedNftTokenId, bytes memory returndataOwnedNftTokenId) = boostContractAddr.staticcall(abi.encodeWithSignature("tokenOfOwnerByIndex(address,uint256)", sender, index));
            if (!successOwnedNftTokenId) {
                revert("Cannot read owned NFT at a given index.");
            }
            uint256 ownedNftTokenId = abi.decode(returndataOwnedNftTokenId, (uint256));
            (bool successTokenUri, bytes memory returndataTokenUri) = boostContractAddr.staticcall(abi.encodeWithSignature("tokenURI(uint256)", ownedNftTokenId));
            if (!successTokenUri) {
                revert("Cannot read token URI of the given ID.");
            }

            if (desiredHaashedTokenUri == keccak256(bytes(abi.decode(returndataTokenUri, (string))))) {
               // 3. find the tokenId, perform safeTransferFrom
                (bool successStakeNft, ) = boostContractAddr.call(abi.encodeWithSignature("safeTransferFrom(address,address,uint256)", sender, recipient, ownedNftTokenId));
                if (!successStakeNft) {
                    revert("Cannot stake the NFT");
                }
                break;
            }
        }


        if (index >= ownedNftBalance) {
            revert("Failed to find the owned NFT");
        }
    }
}
