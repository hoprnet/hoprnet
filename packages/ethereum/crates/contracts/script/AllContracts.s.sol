// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Script.sol";
import "../test/utils/Deploy.sol";
import "./utils/EnvironmentConfig.s.sol";

contract DeployAllContractsScript is Script, EnvironmentConfig, ERC1820RegistryFixture {
    function run() external {
        // 1. Environment check
        // get envirionment of the script
        getEnvrionment();
        // read records of deployed files
        readCurrentEnvironment();
        // Halt if ERC1820Registry has not been deployed.
        mustHaveErc1820Registry();

        // 2. Get deployer private key
        uint256 deployerPrivateKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        vm.startBroadcast(deployerPrivateKey);

        // 3. Deploy
        // 3.1. HoprToken Contract
        // Only deploy Token contract when no deployed one is detected. 
        // E.g. always in development envirionment, or should a new token contract be introduced in staging/production.
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.hoprTokenContractAddress)) {
            // deploy token contract
            currentEnvironmentDetail.hoprTokenContractAddress = deployCode("HoprToken.sol");
        }

        // 3.2. HoprChannels Contract
        // Only deploy Channels contract when no deployed one is detected. 
        // E.g. always in development envirionment, or should a new channel contract be introduced in staging/production per meta environment. 
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.hoprChannelsContractAddress)) {
            // deploy channels contract
            uint256 closure = currentEnvironmentType == EnvironmentType.DEVELOPMENT ? 15 : 5 * 60;
            currentEnvironmentDetail.hoprChannelsContractAddress = deployCode("HoprChannels.sol", abi.encode(currentEnvironmentDetail.hoprTokenContractAddress, closure));
        }

        // 3.3. xHoprToken Contract
        // Only deploy Token contract when no deployed one is detected. 
        // E.g. always in development envirionment, or should a new token contract be introduced in staging. 
        // Production contract should remain 0xD057604A14982FE8D88c5fC25Aac3267eA142a08 TODO: Consider force check on this address
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.xhoprTokenContractAddress)) {
            // deploy xtoken contract
            currentEnvironmentDetail.xhoprTokenContractAddress = deployCode("ERC677Mock.sol");
        }
        
        // 3.4. HoprBoost Contract
        // Only deploy Boost contract when no deployed one is detected. 
        // E.g. always in development envirionment, or should a new token contract be introduced in staging. 
        // Production contract should remain 0x43d13D7B83607F14335cF2cB75E87dA369D056c7 TODO: Consider force check on this address
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.hoprBoostContractAddress)) {
            // deploy boost contract
            currentEnvironmentDetail.hoprBoostContractAddress = deployCode("HoprBoost.sol", abi.encode(deployerAddress, ""));
        }

        // 3.5. HoprStake Contract
        // Only deply HoprStake contract (of the latest season) when no deployed one is detected.
        // E.g. always in development environment, or should a new stake contract be introduced in staging.
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.stakeContractAddress)) {
            // build the staking season artifact name, based on the stake season number specified in the contract-addresses.json
            string memory stakeArtifactName = string(abi.encodePacked("HoprStakeSeason", vm.toString(currentEnvironmentDetail.stakeSeason), ".sol"));
            // deploy stake contract
            currentEnvironmentDetail.stakeContractAddress = deployCode(stakeArtifactName, abi.encode(deployerAddress, currentEnvironmentDetail.hoprBoostContractAddress, currentEnvironmentDetail.xhoprTokenContractAddress, currentEnvironmentDetail.hoprTokenContractAddress));
        }

        // 3.6. NetworkRegistryProxy Contract
        // Only deploy NetworkRegistryProxy contract when no deployed one is detected.
        // E.g. Always in development environment, or should a new NetworkRegistryProxy contract be introduced in staging/production
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
            // deploy DummyProxy in DEVELOPMENT envirionment
            currentEnvironmentDetail.networkRegistryProxyContractAddress = deployCode("HoprDummyProxyForNetworkRegistry.sol", abi.encode(deployerAddress));
        } else if (!isValidAddress(currentEnvironmentDetail.networkRegistryProxyContractAddress)) {
            // deploy StakingProxy in other envrionment types, if no proxy contract is given.
            currentEnvironmentDetail.networkRegistryProxyContractAddress = deployCode("HoprStakingProxyForNetworkRegistry.sol", abi.encode(currentEnvironmentDetail.stakeContractAddress, deployerAddress, 1000 ether));
            // If needed, add `eligibleNftTypeAndRank` TODO: Only execute this transaction when NR accepts accounts with staking amount above certain threshold
            // If needed, add `specialNftTypeAndRank` besides `Network_registry` NFT (index. 26) (`developer` and `community`) TODO: extend this array if more NR NFTs are issued
            (bool successOwnerBatchAddSpecialNftTypeAndRank, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])", [26, 26], ["developer", "community"], [type(uint256).max, 1]));
            if (!successOwnerBatchAddSpecialNftTypeAndRank) {
                revert("Cannot ownerBatchAddSpecialNftTypeAndRank");
            }
        } else {
            // When a NetworkRegistryProxy contract is provided, check if its `stakeContract` matches with the latest stakeContractAddress. 
            (bool successReadStakeContract, bytes memory returndataStakeContract) = currentEnvironmentDetail.networkRegistryProxyContractAddress.staticcall(abi.encodeWithSignature("stakeContract()"));
            if (!successReadStakeContract) {
                revert("Cannot read stakeContract");
            }
            address linkedStakeContract = abi.decode(returndataStakeContract, (address));
            // Check if the current sender is NetworkRegistryProxy owner. 
            (bool successReadProxyOwner, bytes memory returndataProxyOwner) = currentEnvironmentDetail.networkRegistryProxyContractAddress.staticcall(abi.encodeWithSignature("owner()"));
            if (!successReadProxyOwner) {
                revert("Cannot read owner");
            }
            address proxyOwner = abi.decode(returndataProxyOwner, (address));
            // When a mismatch is deteced and the deployer (transaction sender) is the owner, update the `stakeContract` with the latest staking contract address
            if (linkedStakeContract != currentEnvironmentDetail.stakeContractAddress && proxyOwner == deployerAddress) {
                (bool successUpdateStakeContract, ) = currentEnvironmentDetail.networkRegistryProxyContractAddress.call(abi.encodeWithSignature("updateStakeContract(address)", currentEnvironmentDetail.stakeContractAddress));
                if (!successUpdateStakeContract) {
                    revert("Cannot updateStakeContract");
                }
            }
        }

        // 3.7. NetworkRegistry Contract
        // Only deploy NetworkRegistrycontract when no deployed one is detected.
        // E.g. Always in development environment, or should a new NetworkRegistryProxy contract be introduced in staging/production
        if (currentEnvironmentType == EnvironmentType.DEVELOPMENT || !isValidAddress(currentEnvironmentDetail.networkRegistryProxyContractAddress)) {
            // deploy NetworkRegistry contract
            currentEnvironmentDetail.networkRegistryContractAddress = deployCode("HoprNetworkRegistry.sol", abi.encode(currentEnvironmentDetail.networkRegistryProxyContractAddress, deployerAddress));
            // NetworkRegistry should be enabled (default behavior) in staging/production, and disabled in development
            if (currentEnvironmentType == EnvironmentType.DEVELOPMENT) {
                (bool successDisableRegistry, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("disableRegistry()"));
                if (!successDisableRegistry) {
                    revert("Cannot disableRegistry");
                }
            }
        } else {
            // When a NetworkRegistry contract is provided, check if its `requirementImplementation` matches with the latest NetworkRegistryProxy. 
            (bool successReadRequirementImplementation, bytes memory returndataRequirementImplementation) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("requirementImplementation()"));
            if (!successReadRequirementImplementation) {
                revert("Cannot read RequirementImplementation");
            }
            address requirementImplementation = abi.decode(returndataRequirementImplementation, (address));
            // Check if the current sender is NetworkRegistry owner. 
            (bool successReadOwner, bytes memory returndataOwner) = currentEnvironmentDetail.networkRegistryContractAddress.staticcall(abi.encodeWithSignature("owner()"));
            if (!successReadOwner) {
                revert("Cannot read NetworkRegistry contract owner");
            }
            address networkRegistryOwner = abi.decode(returndataOwner, (address));
            // When a mismatch is deteced and the deployer (transaction sender) is the owner, update the `requirementImplementation` with the latest NetworkRegistryProxy address
            if (requirementImplementation != currentEnvironmentDetail.networkRegistryProxyContractAddress && networkRegistryOwner == deployerAddress) {
                (bool successUpdateImplementation, ) = currentEnvironmentDetail.networkRegistryContractAddress.call(abi.encodeWithSignature("updateRequirementImplementation(address)", currentEnvironmentDetail.networkRegistryProxyContractAddress));
                if (!successUpdateImplementation) {
                    revert("Cannot updateRequirementImplementation");
                }
            }
        }

        // write to file
        vm.stopBroadcast();

        // FIXME: to write to a json file
        displayCurrentEnvironmentDetail();
    }
}
