pragma solidity >=0.8.0 <0.9.0;

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
        EnvironmentType environmentType;
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
        currentEnvironmentType = parseEnvironmentTypeFromString(profile);
    }

    function readProtocolConfig() internal view returns (string memory json) {
        string memory root = vm.projectRoot();
        // TODO: Check path to protocol-config.json
        string memory path = string(abi.encodePacked(root, "/contracts-addresses.json"));
        json = vm.readFile(path);
    }

    function readEnvironment(string memory _environmentName) internal {
        string memory json = readProtocolConfig();
        bytes memory levelToEnvironmentConfig = abi.encodePacked(".environments.", _environmentName);

        // read all the contract addresses from contracts-addresses.json. This way ensures that the order of attributes does not affect parsing
        EnvironmentType envType = parseEnvironmentTypeFromString(json.readString(string(abi.encodePacked(levelToEnvironmentConfig, ".environment_type"))));
        uint256 stakeSeasonNum = json.readUint(string(abi.encodePacked(levelToEnvironmentConfig, ".stake_season")));
        address tokenAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".token_contract_address")));
        address channelAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".channels_contract_address")));
        address xhoprAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".xhopr_contract_address")));
        address boostAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".boost_contract_address")));
        address stakeAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".stake_contract_address")));
        address networkRegistryProxyAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".network_registry_proxy_contract_address")));
        address networkRegistryAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, ".network_registry_contract_address")));

        currentEnvironmentDetail = EnvironmentDetail({
            environmentType: envType,
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
    function displayEnvironmentDetail(string memory filePath, EnvironmentDetail memory envDetail) internal {
        vm.writeLine(filePath, string(abi.encodePacked('"environment_type": "', parseEnvironmentTypeToString(envDetail.environmentType), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"stake_season": ', vm.toString(envDetail.stakeSeason), ',')));
        vm.writeLine(filePath, string(abi.encodePacked('"token_contract_address": "', vm.toString(envDetail.hoprTokenContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"channels_contract_address": "', vm.toString(envDetail.hoprChannelsContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"xhopr_contract_address": "', vm.toString(envDetail.xhoprTokenContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"boost_contract_address": "', vm.toString(envDetail.hoprBoostContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"stake_contract_address": "', vm.toString(envDetail.stakeContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"network_registry_proxy_contract_address": "', vm.toString(envDetail.networkRegistryProxyContractAddress), '",')));
        vm.writeLine(filePath, string(abi.encodePacked('"network_registry_contract_address": "', vm.toString(envDetail.networkRegistryContractAddress), '"')));
    }
    // FIXME: remove this temporary method
    function displayCurrentEnvironmentDetail() internal {
        displayEnvironmentDetail("test.txt", currentEnvironmentDetail);
    }

    function isValidAddress(address addr) public pure returns (bool) {
        return addr == address(32) || addr == address(0) ? false : true;
    }

    function parseEnvironmentTypeFromString(string memory environmentType) public pure returns (EnvironmentType) {
        if (keccak256(bytes(environmentType)) == keccak256(bytes("production"))) {
            return EnvironmentType.PRODUCTION;
        } else if (keccak256(bytes(environmentType)) == keccak256(bytes("staging"))) {
            return EnvironmentType.STAGING;
        } else {
            return EnvironmentType.DEVELOPMENT;
        }
    }

    function parseEnvironmentTypeToString(EnvironmentType environmentType) public pure returns (string memory) {
        if (environmentType == EnvironmentType.PRODUCTION) {
            return "production";
        } else if (environmentType == EnvironmentType.STAGING) {
            return "staging";
        } else {
            return "development";
        }
    }
}