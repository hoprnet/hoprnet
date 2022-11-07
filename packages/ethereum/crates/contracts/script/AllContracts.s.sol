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
        // write to file
        vm.stopBroadcast();

        // FIXME: to write to a json file
        displayCurrentEnvironmentDetail();
    }
}
