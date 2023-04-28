// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;
pragma abicoder v2;

import 'forge-std/Script.sol';
import '../test/utils/ERC1820Registry.sol';
import '../test/utils/PermittableToken.sol';
import './utils/NetworkConfig.s.sol';
import './utils/BoostUtilsLib.sol';

contract DeployAllContractsScript is Script, NetworkConfig, ERC1820RegistryFixtureTest, PermittableTokenFixtureTest {
  using BoostUtilsLib for address;
  bool private isHoprChannelsDeployed;
  bool private isHoprNetworkRegistryDeployed;

  function run() external {
    // 1. Network check
    // get envirionment of the script
    getNetwork();
    // read records of deployed files
    readCurrentNetwork();
    // Halt if ERC1820Registry has not been deployed.
    mustHaveErc1820Registry();
    emit log_string(string(abi.encodePacked('Deploying in ', currentNetworkId)));

    // 2. Get deployer private key
    uint256 deployerPrivateKey = vm.envUint('DEPLOYER_PRIVATE_KEY');
    address deployerAddress = vm.addr(deployerPrivateKey);
    vm.startBroadcast(deployerPrivateKey);

    // 3. Deploy
    // 3.1. HoprToken Contract
    // Only deploy Token contract when no deployed one is detected.
    // E.g. always in local envirionment, or should a new token contract be introduced in development/staging/production.
    if (
      currentEnvironmentType == EnvironmentType.LOCAL || !isValidAddress(currentNetworkDetail.hoprTokenContractAddress)
    ) {
      // deploy token contract
      currentNetworkDetail.hoprTokenContractAddress = deployCode('HoprToken.sol');
      // grant deployer minter role
      (bool successGrantMinterRole, ) = currentNetworkDetail.hoprTokenContractAddress.call(
        abi.encodeWithSignature('grantRole(bytes32,address)', MINTER_ROLE, deployerAddress)
      );
      if (!successGrantMinterRole) {
        emit log_string('Cannot grantMinterRole');
      }
      // mint some tokens to the deployer
      (bool successMintTokens, ) = currentNetworkDetail.hoprTokenContractAddress.call(
        abi.encodeWithSignature('mint(address,uint256,bytes,bytes)', deployerAddress, 130000000 ether, hex'00', hex'00')
      );
      if (!successMintTokens) {
        emit log_string('Cannot mint tokens');
      }
    }

    // 3.2. HoprChannels Contract
    // Only deploy Channels contract when no deployed one is detected.
    // E.g. always in local envirionment, or should a new channel contract be introduced in development/staging/production per meta environment.
    if (
      currentEnvironmentType == EnvironmentType.LOCAL ||
      !isValidAddress(currentNetworkDetail.hoprChannelsContractAddress)
    ) {
      // deploy channels contract
      uint256 closure = currentEnvironmentType == EnvironmentType.LOCAL ? 15 : 5 * 60;
      currentNetworkDetail.hoprChannelsContractAddress = deployCode(
        'HoprChannels.sol',
        abi.encode(currentNetworkDetail.hoprTokenContractAddress, closure)
      );
      isHoprChannelsDeployed = true;
    }

    // 3.3. xHoprToken Contract
    // Only deploy Token contract when no deployed one is detected.
    // E.g. always in local envirionment, or should a new token contract be introduced in development/staging.
    // Production contract should remain 0xD057604A14982FE8D88c5fC25Aac3267eA142a08 TODO: Consider force check on this address
    if (currentEnvironmentType == EnvironmentType.LOCAL) {
      // Use the same contract address as in production (HOPR token on xDAI)
      currentNetworkDetail.xhoprTokenContractAddress = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08;
      // set deployed code of permittable token to the address. Set owner and bridge contract of the permittable token
      etchPermittableTokenAt(currentNetworkDetail.xhoprTokenContractAddress);
      // mint 5000000 ether tokens to the deployer by modifying the storage
      (bool successMintXTokensInDeployment, ) = currentNetworkDetail.xhoprTokenContractAddress.call(
        abi.encodeWithSignature('mint(address,uint256)', deployerAddress, 5000000 ether)
      );
      if (!successMintXTokensInDeployment) {
        emit log_string('Cannot mint xHOPR tokens in deployment');
      }
    } else if (!isValidAddress(currentNetworkDetail.xhoprTokenContractAddress)) {
      // deploy xtoken contract
      currentNetworkDetail.xhoprTokenContractAddress = deployCode('ERC677Mock.sol');
      // mint 5 million xHOPR tokens to the deployer
      bytes memory builtMintXHoprPayload = buildXHoprBatchMintInternal(deployerAddress); // This payload is built because default abi.encode returns different value (no size info) when array is static.
      (bool successMintXTokens, ) = currentNetworkDetail.xhoprTokenContractAddress.call(builtMintXHoprPayload);
      if (!successMintXTokens) {
        emit log_string('Cannot mint xHOPR tokens');
      }
    }

    // 3.4. HoprBoost Contract
    // Only deploy Boost contract when no deployed one is detected.
    // E.g. always in local envirionment, or should a new token contract be introduced in development/staging.
    // Production contract should remain 0x43d13D7B83607F14335cF2cB75E87dA369D056c7 TODO: Consider force check on this address
    if (
      currentEnvironmentType == EnvironmentType.LOCAL || !isValidAddress(currentNetworkDetail.hoprBoostContractAddress)
    ) {
      // deploy boost contract
      currentNetworkDetail.hoprBoostContractAddress = deployCode(
        'HoprBoost.sol',
        abi.encode(deployerAddress, 'https://')
      );
    }

    // 3.5. HoprStake Contract
    // Only deply HoprStake contract (of the latest season) when no deployed one is detected.
    // E.g. always in local environment, or should a new stake contract be introduced in development/staging.
    if (currentEnvironmentType == EnvironmentType.LOCAL || !isValidAddress(currentNetworkDetail.stakeContractAddress)) {
      // build the staking season artifact name, based on the stake season number specified in the contract-addresses.json
      string memory stakeArtifactName = string(
        abi.encodePacked('HoprStakeSeason', vm.toString(currentNetworkDetail.stakeSeason), '.sol')
      );
      // deploy stake contract
      currentNetworkDetail.stakeContractAddress = deployCode(
        stakeArtifactName,
        abi.encode(
          deployerAddress,
          currentNetworkDetail.hoprBoostContractAddress,
          currentNetworkDetail.xhoprTokenContractAddress,
          currentNetworkDetail.hoprTokenContractAddress
        )
      );
    }

    // 3.6. NetworkRegistryProxy Contract
    // Only deploy NetworkRegistryProxy contract when no deployed one is detected.
    // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in development/staging/production
    if (currentEnvironmentType == EnvironmentType.LOCAL) {
      // deploy DummyProxy in LOCAL envirionment
      currentNetworkDetail.networkRegistryProxyContractAddress = deployCode(
        'HoprDummyProxyForNetworkRegistry.sol',
        abi.encode(deployerAddress)
      );
      isHoprNetworkRegistryDeployed = true;
    } else if (!isValidAddress(currentNetworkDetail.networkRegistryProxyContractAddress)) {
      // deploy StakingProxy in other environment types, if no proxy contract is given.
      currentNetworkDetail.networkRegistryProxyContractAddress = deployCode(
        'HoprStakingProxyForNetworkRegistry.sol',
        abi.encode(currentNetworkDetail.stakeContractAddress, deployerAddress, 1000 ether)
      );
      isHoprNetworkRegistryDeployed = true;

      // TODO: If needed, add `eligibleNftTypeAndRank`. Only execute this transaction when NR accepts accounts with staking amount above certain threshold
      // Add `Network_registry` NFT (index. 26) (`developer` and `community`) into `specialNftTypeAndRank` TODO: extend this array if more NR NFTs are to be included
      bytes memory builtProxyPayload = buildBatchRegisterSpecialNrNft(); // This payload is built because default abi.encode returns different value (no size info) when array is static.
      (bool successOwnerBatchAddSpecialNftTypeAndRank, ) = currentNetworkDetail
        .networkRegistryProxyContractAddress
        .call(builtProxyPayload);
      if (!successOwnerBatchAddSpecialNftTypeAndRank) {
        emit log_string('Cannot ownerBatchAddSpecialNftTypeAndRank');
        emit log_bytes(builtProxyPayload);
      }
    } else {
      // When a NetworkRegistryProxy contract is provided, check if its `stakeContract` matches with the latest stakeContractAddress.
      (bool successReadStakeContract, bytes memory returndataStakeContract) = currentNetworkDetail
        .networkRegistryProxyContractAddress
        .staticcall(abi.encodeWithSignature('stakeContract()'));
      if (!successReadStakeContract) {
        emit log_string('Cannot read stakeContract');
      }
      address linkedStakeContract = abi.decode(returndataStakeContract, (address));
      // Check if the current sender is NetworkRegistryProxy owner.
      (bool successReadProxyOwner, bytes memory returndataProxyOwner) = currentNetworkDetail
        .networkRegistryProxyContractAddress
        .staticcall(abi.encodeWithSignature('owner()'));
      if (!successReadProxyOwner) {
        emit log_string('Cannot read owner');
      }
      address proxyOwner = abi.decode(returndataProxyOwner, (address));
      // // When a mismatch is detected and the deployer (transaction sender) is the owner,
      // // update the `stakeContract` with the latest staking contract address, if the mew staking contract has started
      (
        bool successReadCurrentStakeContractStartTime,
        bytes memory returndataCurrentStakeContractStartTime
      ) = currentNetworkDetail.stakeContractAddress.staticcall(abi.encodeWithSignature('PROGRAM_START()'));
      if (!successReadCurrentStakeContractStartTime) {
        emit log_string('Cannot read successReadCurrentStakeContractStartTime');
      }
      uint256 currentStakeStartTime = abi.decode(returndataCurrentStakeContractStartTime, (uint256));
      if (
        linkedStakeContract != currentNetworkDetail.stakeContractAddress &&
        proxyOwner == deployerAddress &&
        currentStakeStartTime <= block.timestamp
      ) {
        (bool successUpdateStakeContract, ) = currentNetworkDetail.networkRegistryProxyContractAddress.call(
          abi.encodeWithSignature('updateStakeContract(address)', currentNetworkDetail.stakeContractAddress)
        );
        if (!successUpdateStakeContract) {
          emit log_string('Cannot updateStakeContract');
        }
      }
    }

    // 3.7. NetworkRegistry Contract
    // Only deploy NetworkRegistrycontract when no deployed one is detected.
    // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in development/staging/production
    if (
      currentEnvironmentType == EnvironmentType.LOCAL ||
      !isValidAddress(currentNetworkDetail.networkRegistryContractAddress)
    ) {
      // deploy NetworkRegistry contract
      currentNetworkDetail.networkRegistryContractAddress = deployCode(
        'HoprNetworkRegistry.sol',
        abi.encode(currentNetworkDetail.networkRegistryProxyContractAddress, deployerAddress)
      );
      // NetworkRegistry should be enabled (default behavior) in staging/production, and disabled in development
      if (currentEnvironmentType == EnvironmentType.LOCAL) {
        (bool successDisableRegistry, ) = currentNetworkDetail.networkRegistryContractAddress.call(
          abi.encodeWithSignature('disableRegistry()')
        );
        if (!successDisableRegistry) {
          emit log_string('Cannot disableRegistry');
        }
      }
    } else {
      // When a NetworkRegistry contract is provided, check if its `requirementImplementation` matches with the latest NetworkRegistryProxy.
      (
        bool successReadRequirementImplementation,
        bytes memory returndataRequirementImplementation
      ) = currentNetworkDetail.networkRegistryContractAddress.staticcall(
          abi.encodeWithSignature('requirementImplementation()')
        );
      if (!successReadRequirementImplementation) {
        emit log_string('Cannot read RequirementImplementation');
      }
      address requirementImplementation = abi.decode(returndataRequirementImplementation, (address));
      // Check if the current sender is NetworkRegistry owner.
      (bool successReadOwner, bytes memory returndataOwner) = currentNetworkDetail
        .networkRegistryContractAddress
        .staticcall(abi.encodeWithSignature('owner()'));
      if (!successReadOwner) {
        emit log_string('Cannot read NetworkRegistry contract owner');
      }
      address networkRegistryOwner = abi.decode(returndataOwner, (address));
      // When a mismatch is deteced and the deployer (transaction sender) is the owner, update the `requirementImplementation` with the latest NetworkRegistryProxy address
      if (
        requirementImplementation != currentNetworkDetail.networkRegistryProxyContractAddress &&
        networkRegistryOwner == deployerAddress
      ) {
        (bool successUpdateImplementation, ) = currentNetworkDetail.networkRegistryContractAddress.call(
          abi.encodeWithSignature(
            'updateRequirementImplementation(address)',
            currentNetworkDetail.networkRegistryProxyContractAddress
          )
        );
        if (!successUpdateImplementation) {
          emit log_string('Cannot updateRequirementImplementation');
        }
      }
    }

    // 4. Batch mint Network_registry NFTs in local/development/staging envirionment
    // Ensure a "Network_registry" boost type is at the index 26. If not, mint dummy proxies (E.g. "Dummy_1") until index 25 and "Network_registry" at 26
    (bool existAtNetworkRegistryIndex, string memory nameOrError) = currentNetworkDetail
      .hoprBoostContractAddress
      .getBoostTypeAtIndex(NETWORK_REGISTRY_NFT_INDEX);
    if (existAtNetworkRegistryIndex && keccak256(bytes(nameOrError)) != NETWORK_REGISTRY_TYPE_HASH) {
      // when type at place is not right
      revert('NFT type mismatch. Need to redeploy Boost contract');
    }
    // mint dummy NFTs (1..25)
    for (uint256 index = 1; index < NETWORK_REGISTRY_NFT_INDEX; index++) {
      // boost type is one-based index
      (bool existAtIndex, ) = currentNetworkDetail.hoprBoostContractAddress.getBoostTypeAtIndex(index);
      if (existAtIndex) {
        continue;
      } else {
        // mint a dummy type
        (bool successMintDummyNft, ) = currentNetworkDetail.hoprBoostContractAddress.call(
          abi.encodeWithSignature(
            'mint(address,string,string,uint256,uint256)',
            deployerAddress,
            string(abi.encode(DUMMY_TYPE_PREFIX, vm.toString(index))),
            DUMMY_TYPE_PREFIX,
            0,
            0
          )
        );
        if (!successMintDummyNft) {
          revert('Error in minting dummy nfts');
        }
      }
    }
    // mint Network_registry type (except for production)
    if (currentEnvironmentType != EnvironmentType.PRODUCTION) {
      (bytes memory builtNftBatchMintPayload1, bytes memory builtNftBatchMintPayload2) = buildNftBatchMintInternal(
        deployerAddress,
        DEV_BANK_ADDRESS
      ); // This payload is built because default abi.encode returns different value (no size info) when array is static.
      (bool successBatchMint1, ) = currentNetworkDetail.hoprBoostContractAddress.call(builtNftBatchMintPayload1);
      (bool successBatchMint2, ) = currentNetworkDetail.hoprBoostContractAddress.call(builtNftBatchMintPayload2);
      if (!successBatchMint1 || !successBatchMint2) {
        revert('Error in minting Network_registry in batches');
      }
    }

    // if both HoprChannels and HoprNetworkRegistry contracts are deployed, update the startup block number for indexer
    if (isHoprChannelsDeployed && isHoprNetworkRegistryDeployed) {
      currentNetworkDetail.indexerStartBlockNumber = block.number;
    }

    // broadcast transaction bundle
    vm.stopBroadcast();

    // write to file
    writeCurrentNetwork();
  }

  /**
   * @dev Helper function to build payload for "ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])"
   * By default, it adds `Network_registry` NFT (index. 26) (`developer` and `community`)
   * It's possible to extend this array if more NR NFTs are issued
   */
  function buildBatchRegisterSpecialNrNft() private pure returns (bytes memory) {
    // "Network_registry" type
    uint256[] memory typeIndex = new uint256[](2);
    typeIndex[0] = 26;
    typeIndex[1] = 26;
    // "developer" and "community" rank
    string[] memory ranks = new string[](2);
    ranks[0] = 'developer';
    ranks[1] = 'community';
    // max. number of allowed registration
    uint256[] memory maxAllowedReg = new uint256[](2);
    maxAllowedReg[0] = type(uint256).max;
    maxAllowedReg[1] = 1;

    return
      abi.encodeWithSignature(
        'ownerBatchAddSpecialNftTypeAndRank(uint256[],string[],uint256[])',
        typeIndex,
        ranks,
        maxAllowedReg
      );
  }

  /**
   * @dev Helper function to build payload for "batchMintInternal(address[],uint256)"
   */
  function buildXHoprBatchMintInternal(address addr) private pure returns (bytes memory) {
    address[] memory addrBook = new address[](1);
    addrBook[0] = addr;

    return abi.encodeWithSignature('batchMintInternal(address[],uint256)', addrBook, 5000000 ether);
  }

  /**
   * @dev Helper function to build payload for "batchMint(address[],string,string,uint256,uint256)"
   */
  function buildNftBatchMintInternal(
    address addr1,
    address addr2
  ) private pure returns (bytes memory devPayload, bytes memory communityPayload) {
    address[] memory addrBook = new address[](6);
    addrBook[0] = addr1;
    addrBook[1] = addr1;
    addrBook[2] = addr1;
    addrBook[3] = addr2;
    addrBook[4] = addr2;
    addrBook[5] = addr2;

    devPayload = abi.encodeWithSignature(
      'batchMint(address[],string,string,uint256,uint256)',
      addrBook,
      NETWORK_REGISTRY_TYPE_NAME,
      NETWORK_REGISTRY_RANK1_NAME,
      0,
      0
    );
    communityPayload = abi.encodeWithSignature(
      'batchMint(address[],string,string,uint256,uint256)',
      addrBook,
      NETWORK_REGISTRY_TYPE_NAME,
      NETWORK_REGISTRY_RANK2_NAME,
      0,
      0
    );
  }
}
