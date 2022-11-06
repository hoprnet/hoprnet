pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/StdJson.sol";

/**
 * Get environment_type from the envrionment variable `FOUNDRY_PROFILE` 
 * Get environment_name string from the envrionment variable "ENVIRONMENT_NAME"
 */
contract EnvironmentConfig is Script {
    using stdJson for string;

    enum EnvironmentType {
        DEVELOPMENT,
        STAGING,
        PRODUCTION
    }
    
    struct EnvironmentDetail {
        address hoprTokenContractAddress;
        address hoprChannelsContractAddress;
        address xhoprTokenContractAddress;
        address hoprBoostContractAddress;
        address stakeContractAddress;
        address networkRegistryContractAddress;
        address networkRegistryProxyContractAddress;
    }

    string public currentEnvironmentName;
    EnvironmentType public currentEnvironmentType;
    EnvironmentDetail public currentEnvironmentDetail;

    function getEnvrionment() public {
         // get envirionment of the script
        string memory profile = vm.envString("FOUNDRY_PROFILE");
        currentEnvironmentName = vm.envString("ENVIRONMENT_NAME");
        
        if (keccak256(bytes(profile)) == keccak256(bytes("production"))) {
            currentEnvironmentType = EnvironmentType.PRODUCTION;
        } else if (keccak256(bytes(profile)) == keccak256(bytes("staging"))) {
            currentEnvironmentType = EnvironmentType.STAGING;
        } else {
            currentEnvironmentType = EnvironmentType.DEVELOPMENT;
        }
    }

    function readEnvironment(string memory _environmentName) internal {
        string memory root = vm.projectRoot();
        string memory path = string(abi.encodePacked(root, "/contracts-addresses.json"));
        string memory json = vm.readFile(path);

        // read all the contract addresses from contracts-addresses.json
        address tokenAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprTokenContractAddress")));
        address channelAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprChannelsContractAddress")));
        address xhoprAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".xhoprTokenContractAddress")));
        address boostAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprBoostContractAddress")));
        address stakeAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".stakeContractAddress")));
        address networkRegistryAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".networkRegistryContractAddress")));
        address networkRegistryProxyAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".networkRegistryProxyContractAddress")));

        currentEnvironmentDetail = EnvironmentDetail({
            hoprTokenContractAddress: tokenAddr,
            hoprChannelsContractAddress: channelAddr,
            xhoprTokenContractAddress: xhoprAddr,
            hoprBoostContractAddress: boostAddr,
            stakeContractAddress: stakeAddr,
            networkRegistryContractAddress: networkRegistryAddr,
            networkRegistryProxyContractAddress: networkRegistryProxyAddr
        });

        // FIXME: remove this temporary method
        displayCurrentEnvironmentDetail();
    }

    function readCurrentEnvironment() internal {
        readEnvironment(currentEnvironmentName);
    }

    // use writeJson
    function writeEnvironment() internal {
    }

    // FIXME: remove this temporary method
    function displayCurrentEnvironmentDetail() internal {
        vm.writeLine("test.txt", string(abi.encodePacked("tokenAddr: ", vm.toString(currentEnvironmentDetail.hoprTokenContractAddress), vm.toString(isValidAddress(currentEnvironmentDetail.hoprTokenContractAddress)))));
        vm.writeLine("test.txt", string(abi.encodePacked("channelAddr: ", vm.toString(currentEnvironmentDetail.hoprChannelsContractAddress))));
        vm.writeLine("test.txt", string(abi.encodePacked("xhoprAddr: ", vm.toString(currentEnvironmentDetail.xhoprTokenContractAddress))));
        vm.writeLine("test.txt", string(abi.encodePacked("boostAddr: ", vm.toString(currentEnvironmentDetail.hoprBoostContractAddress))));
        vm.writeLine("test.txt", string(abi.encodePacked("stakeAddr: ", vm.toString(currentEnvironmentDetail.stakeContractAddress))));
        vm.writeLine("test.txt", string(abi.encodePacked("networkRegistryAddr: ", vm.toString(currentEnvironmentDetail.networkRegistryContractAddress))));
        vm.writeLine("test.txt", string(abi.encodePacked("networkRegistryProxyAddr: ", vm.toString(currentEnvironmentDetail.networkRegistryProxyContractAddress))));
    }

    function isValidAddress(address addr) public pure returns (bool) {
        return addr == address(32) || addr == address(0) ? false : true;
    }
}