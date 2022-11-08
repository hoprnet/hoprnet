// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;
pragma abicoder v2;

import "forge-std/Script.sol";
import "forge-std/StdJson.sol";
import "forge-std/Test.sol";
import "./utils/EnvironmentConfig.s.sol";

contract TestReadWriteScript is Script, EnvironmentConfig, Test {
    using stdJson for string;

    function run() external {
        // 1. Environment check
        // get envirionment of the script
        getEnvrionment();
        // read records of deployed files
        readCurrentEnvironment();

        string memory json = readProtocolConfig();
        emit log_string(json);

        EnvironmentDetail memory testLocalEnv = EnvironmentDetail({
            environmentType: EnvironmentType.DEVELOPMENT,
            stakeSeason: 5,
            hoprTokenContractAddress: address(1),
            hoprChannelsContractAddress: address(2),
            xhoprTokenContractAddress: address(3),
            hoprBoostContractAddress: address(4),
            stakeContractAddress: address(5),
            networkRegistryContractAddress: address(6),
            networkRegistryProxyContractAddress: address(7)
        });

        string memory parsedNewEnvDetail = parseEnvironmentDetailToString(testLocalEnv);
        emit log(parsedNewEnvDetail);

        // write to file;
        string memory path = "./contracts-addresses.json";
        // write parsedNewEnvDetail to corresponding key
        vm.writeJson(parsedNewEnvDetail, path, ".environments.localhost");
    }

    function parseEnvironmentDetailToString(EnvironmentDetail memory envDetail) internal returns (string memory) {
        string memory json;
        vm.serializeString(json, "environment_type", parseEnvironmentTypeToString(envDetail.environmentType));
        vm.serializeUint(json, "stake_season", envDetail.stakeSeason);
        vm.serializeAddress(json, "token_contract_address", envDetail.hoprTokenContractAddress);
        vm.serializeAddress(json, "channels_contract_address", envDetail.hoprChannelsContractAddress);
        vm.serializeAddress(json, "xhopr_contract_address", envDetail.xhoprTokenContractAddress);
        vm.serializeAddress(json, "boost_contract_address", envDetail.hoprBoostContractAddress);
        vm.serializeAddress(json, "stake_contract_address", envDetail.stakeContractAddress);
        vm.serializeAddress(json, "network_registry_proxy_contract_address", envDetail.networkRegistryProxyContractAddress);
        vm.serializeAddress(json, "network_registry_contract_address", envDetail.networkRegistryContractAddress);
        return json;
    }
}
