// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std-latest/Script.sol";

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
        address announcements;
        address channelsContractAddress;
        address moduleImplementationAddress;
        address networkRegistryContractAddress;
        address networkRegistryProxyContractAddress;
        address nodeSafeRegistryAddress;
        address nodeStakeV2FactoryAddress;
        address ticketPriceOracleContractAddress;
        address tokenContractAddress;
    }

    struct NetworkDetailIntermediate {
        Addresses addresses;
        string environmentType;
        uint256 indexerStartBlockNumber;
    }

    struct NetworkDetail {
        Addresses addresses;
        EnvironmentType environmentType;
        uint256 indexerStartBlockNumber;
    }

    // Deployed contract addresses
    // address constant PROD_WXHOPR_TOKEN_CONTRACT_ADDRESS = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1;
    // TODO: this contract is not necessarily the "HoprToken" contract used in releases
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

    function readNetwork(string memory networkName) internal returns (NetworkDetail memory networkDetail) {
        string memory json = vm.readFile(pathToDeploymentFile);
        bytes memory networkDetailPath = abi.encodePacked(".networks.", networkName);

        bytes memory detailRaw = json.parseRaw(string(networkDetailPath));

        // as long as the alphabetical order of the keys in the JSON file is consistent, the ABI encoding will be
        NetworkDetailIntermediate memory networkDetailIntermediate = abi.decode(detailRaw, (NetworkDetailIntermediate));

        networkDetail = NetworkDetail(
            networkDetailIntermediate.addresses,
            parseEnvironmentTypeFromString(networkDetailIntermediate.environmentType),
            networkDetailIntermediate.indexerStartBlockNumber
        );
    }

    function readCurrentNetwork() internal {
        currentNetworkDetail = readNetwork(currentNetworkId);
    }

    function writeNetwork(string memory networkName, NetworkDetail memory networkDetail) internal {
        // write parsedNewEnvDetail to corresponding key
        string memory configKey = string(abi.encodePacked(".networks.", networkName));
        string memory configKeyAddresses = string(abi.encodePacked(".networks.", networkName, ".addresses"));

        // the keys must be unique because they are stored in shared memory
        string memory obj = string(abi.encodePacked("obj-", networkName));
        string memory addresses = string(abi.encodePacked("addresses-", networkName));

        addresses.serialize("token", networkDetail.addresses.tokenContractAddress);
        addresses.serialize("channels", networkDetail.addresses.channelsContractAddress);
        addresses.serialize("node_stake_v2_factory", networkDetail.addresses.nodeStakeV2FactoryAddress);
        addresses.serialize("module_implementation", networkDetail.addresses.moduleImplementationAddress);
        addresses.serialize("node_safe_registry", networkDetail.addresses.nodeSafeRegistryAddress);
        addresses.serialize("network_registry_proxy", networkDetail.addresses.networkRegistryProxyContractAddress);
        addresses.serialize("ticket_price_oracle", networkDetail.addresses.ticketPriceOracleContractAddress);
        addresses.serialize("announcements", networkDetail.addresses.announcements);
        addresses = addresses.serialize("network_registry", networkDetail.addresses.networkRegistryContractAddress);

        obj.serialize("environment_type", parseEnvironmentTypeToString(networkDetail.environmentType));
        obj.serialize("indexer_start_block_number", networkDetail.indexerStartBlockNumber);
        obj = obj.serialize("addresses", addresses);

        vm.writeJson(obj, pathToDeploymentFile, configKey);
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
