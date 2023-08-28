// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Script.sol";
import "forge-std/StdJson.sol";

/**
 * Get environment_type from the environment variable `FOUNDRY_PROFILE`
 * Get network string from the environment variable "NETWORK"
 */
contract NetworkConfig is Script {
    using stdJson for string;

    enum EnvironmentType {
        LOCAL,
        DEVELOPMENT,
        STAGING,
        PRODUCTION
    }

    struct Addresses {
        address tokenContractAddress;
        address channelsContractAddress;
        address nodeStakeV2FactoryAddress;
        address moduleImplementationAddress;
        address nodeSafeRegistryAddress;
        address networkRegistryContractAddress;
        address networkRegistryProxyContractAddress;
        address ticketPriceOracleContractAddress;
        address announcements;
    }

    struct NetworkDetail {
        EnvironmentType environmentType;
        uint256 indexerStartBlockNumber;
        Addresses addresses;
    }

    // Deployed contract addresses
    // address constant PROD_WXHOPR_TOKEN_CONTRACT_ADDRESS = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1; // TODO: this
    // contract is not necessarily the "HoprToken" contract used in releases
    bytes32 public constant DEFAULT_ADMIN_ROLE = 0x00;
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant MANAGER_ROLE = keccak256("MANAGER_ROLE");
    address public constant DEV_BANK_ADDRESS = 0x2402da10A6172ED018AEEa22CA60EDe1F766655C;
    address public constant COMM_MULTISIG_ADDRESS = 0xD9a00176Cf49dFB9cA3Ef61805a2850F45Cb1D05;
    address public constant PRODUCT_MULTISIG_ADDRESS = 0xD720099cBC14e669695EaE0708E6Ca614B387921; // only used in
        // "stake_hub_test" network
    // CORE's deployer is the caller, therefore not in this array
    address[3] public PRODUCT_TEAM_MANAGER_ADDRESSES = [
        0x01BFbCB6A2924b083969ce6237AdBbF3BFa7De13, // RPCh staging
        0xDCcC4a8ee2BF3CaF5a4AB1cDBa1ee7cc04E324Dd, // RPCh production
        0x529F995739C9C425CECeE9deF78e95CB07887565 // CT
    ];

    string public currentNetworkId;
    EnvironmentType public currentEnvironmentType;
    NetworkDetail public currentNetworkDetail;

    string public pathToDeploymentFile = string(abi.encodePacked(vm.projectRoot(), "/contracts-addresses.json"));

    function getNetwork() public {
        // get environment of the script
        string memory profile = vm.envString("FOUNDRY_PROFILE");
        currentNetworkId = vm.envString("NETWORK");
        currentEnvironmentType = parseEnvironmentTypeFromString(profile);
    }

    function readNetwork(string memory _networkName) internal returns (NetworkDetail memory networkDetail) {
        string memory json = vm.readFile(pathToDeploymentFile);
        bytes memory levelToNetworkConfig = abi.encodePacked(".networks.", _networkName);

        // read all the contract addresses from contracts-addresses.json. This way ensures that the order of attributes
        // does not affect parsing
        EnvironmentType envType = parseEnvironmentTypeFromString(
            json.readString(string(abi.encodePacked(levelToNetworkConfig, ".environment_type")))
        );
        uint256 indexerStartBlkNum =
            json.readUint(string(abi.encodePacked(levelToNetworkConfig, ".indexer_start_block_number")));

        bytes memory addresses = abi.encodePacked(levelToNetworkConfig, ".addresses");

        // console2.log_string(addresses);

        address tokenAddr = json.readAddress(string(abi.encodePacked(addresses, ".token")));
        address channelAddr = json.readAddress(string(abi.encodePacked(addresses, ".channels")));
        address nodeStakeV2FactoryAddr = json.readAddress(string(abi.encodePacked(addresses, ".node_stake_v2_factory")));
        address moduleImplementationAddr =
            json.readAddress(string(abi.encodePacked(addresses, ".module_implementation")));
        address nodeSafeRegistryAddr = json.readAddress(string(abi.encodePacked(addresses, ".node_safe_registry")));
        address networkRegistryProxyAddr =
            json.readAddress(string(abi.encodePacked(addresses, ".network_registry_proxy")));
        address networkRegistryAddr = json.readAddress(string(abi.encodePacked(addresses, ".network_registry")));
        address ticketPriceOracleAddress = json.readAddress(string(abi.encodePacked(addresses, ".ticket_price_oracle")));
        address announcementAdddress = json.readAddress(string(abi.encodePacked(addresses, ".announcements")));

        Addresses memory addressStruct = Addresses({
            tokenContractAddress: tokenAddr,
            channelsContractAddress: channelAddr,
            nodeStakeV2FactoryAddress: nodeStakeV2FactoryAddr,
            moduleImplementationAddress: moduleImplementationAddr,
            nodeSafeRegistryAddress: nodeSafeRegistryAddr,
            networkRegistryContractAddress: networkRegistryAddr,
            networkRegistryProxyContractAddress: networkRegistryProxyAddr,
            ticketPriceOracleContractAddress: ticketPriceOracleAddress,
            announcements: announcementAdddress
        });

        networkDetail = NetworkDetail({
            environmentType: envType,
            indexerStartBlockNumber: indexerStartBlkNum,
            addresses: addressStruct
        });
    }

    function readCurrentNetwork() internal {
        currentNetworkDetail = readNetwork(currentNetworkId);
    }

    function writeNetwork(string memory _networkName, NetworkDetail memory networkDetail) internal {
        // write parsedNewEnvDetail to corresponding key
        string memory configKey = string(abi.encodePacked(".networks.", _networkName));

        // use vm.writeJson to preserve order of JSON properties
        vm.writeJson(
            vm.toString(networkDetail.addresses.tokenContractAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.token"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.channelsContractAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.channels"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.nodeStakeV2FactoryAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.node_stake_v2_factory"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.moduleImplementationAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.module_implementation"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.nodeSafeRegistryAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.node_safe_registry"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.networkRegistryProxyContractAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.network_registry_proxy"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.ticketPriceOracleContractAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.ticket_price_oracle"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.announcements),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.announcements"))
        );
        vm.writeJson(
            vm.toString(networkDetail.addresses.networkRegistryContractAddress),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".addresses.network_registry"))
        );

        vm.writeJson(
            parseEnvironmentTypeToString(networkDetail.environmentType),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".environment_type"))
        );
        vm.writeJson(
            vm.toString(networkDetail.indexerStartBlockNumber),
            pathToDeploymentFile,
            string(abi.encodePacked(configKey, ".indexer_start_block_number"))
        );
    }

    function writeCurrentNetwork() internal {
        // if currentNetworkId is anvil-localhost, update both `anvil-localhost` and `anvil-localhost2`
        if (keccak256(bytes(currentNetworkId)) == keccak256(bytes("anvil-localhost"))) {
            writeNetwork("anvil-localhost2", currentNetworkDetail);
        }
        writeNetwork(currentNetworkId, currentNetworkDetail);
    }

    // FIXME: remove this temporary method
    function displayNetworkDetail(string memory filePath, NetworkDetail memory networkDetail) internal {
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"environment_type": "', parseEnvironmentTypeToString(networkDetail.environmentType), '",'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"indexer_start_block_umber": ', vm.toString(networkDetail.indexerStartBlockNumber), ","
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"token_contract_address": "', vm.toString(networkDetail.addresses.tokenContractAddress), '",'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"channels_contract_address": "', vm.toString(networkDetail.addresses.channelsContractAddress), '",'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"node_stake_v2_factory_address": "',
                    vm.toString(networkDetail.addresses.nodeStakeV2FactoryAddress),
                    '"'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"module_implementation_address": "',
                    vm.toString(networkDetail.addresses.moduleImplementationAddress),
                    '"'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"node_safe_registry_address": "', vm.toString(networkDetail.addresses.nodeSafeRegistryAddress), '"'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"network_registry_proxy_contract_address": "',
                    vm.toString(networkDetail.addresses.networkRegistryProxyContractAddress),
                    '",'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"network_registry_contract_address": "',
                    vm.toString(networkDetail.addresses.networkRegistryContractAddress),
                    '"'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"ticket_price_oracle_contract_address": "',
                    vm.toString(networkDetail.addresses.ticketPriceOracleContractAddress),
                    '",'
                )
            )
        );
        vm.writeLine(
            filePath,
            string(
                abi.encodePacked(
                    '"announcements_contract_address": "', vm.toString(networkDetail.addresses.announcements), '",'
                )
            )
        );
    }

    // FIXME: remove this temporary method
    function displayCurrentNetworkDetail() internal {
        displayNetworkDetail("test.txt", currentNetworkDetail);
    }

    function isValidAddress(address addr) public pure returns (bool) {
        return addr == address(32) || addr == address(0) ? false : true;
    }

    function parseEnvironmentTypeFromString(string memory environmentType) public pure returns (EnvironmentType) {
        if (keccak256(bytes(environmentType)) == keccak256(bytes("production"))) {
            return EnvironmentType.PRODUCTION;
        } else if (keccak256(bytes(environmentType)) == keccak256(bytes("staging"))) {
            return EnvironmentType.STAGING;
        } else if (keccak256(bytes(environmentType)) == keccak256(bytes("development"))) {
            return EnvironmentType.DEVELOPMENT;
        } else {
            return EnvironmentType.LOCAL;
        }
    }

    function parseEnvironmentTypeToString(EnvironmentType environmentType) public pure returns (string memory) {
        if (environmentType == EnvironmentType.PRODUCTION) {
            return "production";
        } else if (environmentType == EnvironmentType.STAGING) {
            return "staging";
        } else if (environmentType == EnvironmentType.DEVELOPMENT) {
            return "development";
        } else {
            return "local";
        }
    }
}
