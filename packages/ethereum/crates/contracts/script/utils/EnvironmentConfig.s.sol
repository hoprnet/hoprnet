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
        uint256 stakeSeason;
        address hoprTokenContractAddress;
        address hoprChannelsContractAddress;
        address xhoprTokenContractAddress;
        address hoprBoostContractAddress;
        address stakeContractAddress;
        address networkRegistryContractAddress;
        address networkRegistryProxyContractAddress;
    }

    // Deployed contract addresses
    // address constant PROD_WXHOPR_TOKEN_CONTRACT_ADDRESS = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1; // TODO: this contract is not necessarily the "HoprToken" contract used in releases
    address constant PROD_XHOPR_TOKEN_CONTRACT_ADDRESS = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08;
    address constant PROD_HOPR_BOOST_CONTRACT_ADDRESS = 0x43d13D7B83607F14335cF2cB75E87dA369D056c7;

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
        uint256 stakeSeasonNum = json.readUint(string(abi.encodePacked(".", _environmentName, ".stakeSeason")));
        address tokenAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprTokenContractAddress")));
        address channelAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprChannelsContractAddress")));
        address xhoprAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".xhoprTokenContractAddress")));
        address boostAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".hoprBoostContractAddress")));
        address stakeAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".stakeContractAddress")));
        address networkRegistryAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".networkRegistryContractAddress")));
        address networkRegistryProxyAddr = json.readAddress(string(abi.encodePacked(".", _environmentName, ".networkRegistryProxyContractAddress")));

        currentEnvironmentDetail = EnvironmentDetail({
            stakeSeason: stakeSeasonNum,
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

    // TODO: use writeJson when https://github.com/foundry-rs/foundry/pull/3595 is merged
    function writeEnvironment() internal {
    }

    // FIXME: remove this temporary method
    function displayCurrentEnvironmentDetail() internal {
        vm.writeLine("test.txt", string(abi.encodePacked("stakeSeasonNum: ", vm.toString(currentEnvironmentDetail.stakeSeason))));
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