// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;
pragma abicoder v2;

import "forge-std/Script.sol";
import "../test/utils/ERC1820Registry.sol";
import "../test/utils/PermittableToken.sol";
import "./utils/NetworkConfig.s.sol";
import "./utils/BoostUtilsLib.sol";

/**
 * @title Deploy all the required contracts in development, staging and production environment
 * @notice In local development environment, ERC1820Registry, Safe deployment singleton, Safe suites should be deployed
 * before running this script.
 * @dev It reads the environment, netork and deployer internal key from env variables
 */
contract DeployAllContractsScript is Script, NetworkConfig, ERC1820RegistryFixtureTest, PermittableTokenFixtureTest {
    using BoostUtilsLib for address;

    bool internal isHoprChannelsDeployed;
    bool internal isHoprNetworkRegistryDeployed;
    address private owner;

    function setUp() public override(ERC1820RegistryFixtureTest) { }

    function run() external {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();
        // Halt if ERC1820Registry has not been deployed.
        mustHaveErc1820Registry();
        emit log_string(string(abi.encodePacked("Deploying in ", currentNetworkId)));
        // get owner of network registry (and its proxy) depending on the network
        if (keccak256(abi.encodePacked(currentNetworkId)) == keccak256(abi.encodePacked("stake_hub_test"))) {
            owner = PRODUCT_MULTISIG_ADDRESS;
        } else {
            owner = COMM_MULTISIG_ADDRESS;
        }

        // 2. Get deployer internal key.
        // Set to default when it's in development environment (uint for
        // 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77_814_517_325_470_205_911_140_941_194_401_928_579_557_062_014_761_831_930_645_393_041_380_819_009_408
            : vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployerAddress = vm.addr(deployerPrivateKey);
        emit log_named_address("deployerAddress", deployerAddress);
        vm.startBroadcast(deployerPrivateKey);

        // 3. Deploy
        // 3.1 HoprNodeStakeFactory
        _deployHoprNodeStakeFactory();

        // 3.2 HoprNodeManagementModule singleton
        _deployHoprNodeManagementModule();

        // 3.3 HoprNodeSafeRegistry
        _deployHoprHoprNodeSafeRegistry();

        // 3.4. HoprToken Contract
        // Only deploy Token contract when no deployed one is detected.
        // E.g. always in local environment, or should a new token contract be introduced in
        // development/staging/production.
        _deployHoprTokenAndMintToAddress(deployerAddress, deployerAddress);

        // 3.5. HoprChannels Contract
        // Only deploy Channels contract when no deployed one is detected.
        // E.g. always in local environment, or should a new channel contract be introduced in
        // development/staging/production per meta environment.
        _deployHoprChannels();

        // 3.6. NetworkRegistryProxy Contract
        // Only deploy NetworkRegistryProxy contract when no deployed one is detected.
        // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in
        // development/staging/production
        _deployNRProxy(deployerAddress);

        // 3.7. NetworkRegistry Contract
        // Only deploy NetworkRegistrycontract when no deployed one is detected.
        // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in
        // development/staging/production
        _deployNetworkRegistry(deployerAddress);

        // 3.8. TicketPriceOracle
        _deployHoprTicketPriceOracle(deployerAddress, 100);

        _deployHoprAnnouncements();

        // 4. update indexerStartBlockNumber
        // if both HoprChannels and HoprNetworkRegistry contracts are deployed, update the startup block number for
        // indexer
        if (isHoprChannelsDeployed && isHoprNetworkRegistryDeployed) {
            currentNetworkDetail.indexerStartBlockNumber = block.number;
        }

        // broadcast transaction bundle
        vm.stopBroadcast();

        // write to file
        writeCurrentNetwork();
    }

    /**
     * @dev deploy node safe factory
     */
    function _deployHoprNodeStakeFactory() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.nodeStakeV2FactoryAddress)
        ) {
            // deploy HoprNodeStakeFactory contract
            currentNetworkDetail.addresses.nodeStakeV2FactoryAddress =
                deployCode("NodeStakeFactory.sol:HoprNodeStakeFactory");
        }
    }

    /**
     * @dev Deploy node management module
     */
    function _deployHoprNodeManagementModule() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.moduleImplementationAddress)
        ) {
            // deploy HoprNodeManagementModule contract
            currentNetworkDetail.addresses.moduleImplementationAddress =
                deployCode("NodeManagementModule.sol:HoprNodeManagementModule");
        }
    }

    /**
     * @dev Deploy node safe registry
     */
    function _deployHoprHoprNodeSafeRegistry() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.nodeSafeRegistryAddress)
        ) {
            // deploy HoprNodeManagementModule contract
            currentNetworkDetail.addresses.nodeSafeRegistryAddress =
                deployCode("NodeSafeRegistry.sol:HoprNodeSafeRegistry");
        }
    }

    /**
     * @dev Deploy hopr token. Set a minter and mint some token to the deployer
     */
    function _deployHoprTokenAndMintToAddress(address deployerAddress, address recipient) internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.tokenContractAddress)
        ) {
            // deploy token contract
            currentNetworkDetail.addresses.tokenContractAddress = deployCode("HoprToken.sol");
            // grant deployer minter role
            (bool successGrantMinterRole,) = currentNetworkDetail.addresses.tokenContractAddress.call(
                abi.encodeWithSignature("grantRole(bytes32,address)", MINTER_ROLE, deployerAddress)
            );
            if (!successGrantMinterRole) {
                emit log_string("Cannot grantMinterRole");
            }
            // mint some tokens to the deployer
            (bool successMintTokens,) = currentNetworkDetail.addresses.tokenContractAddress.call(
                abi.encodeWithSignature(
                    "mint(address,uint256,bytes,bytes)", recipient, 130_000_000 ether, hex"00", hex"00"
                )
            );
            if (!successMintTokens) {
                emit log_string("Cannot mint tokens");
            }
        }
    }

    /**
     * @dev Deploy HoprChannels smart contract and registers NodeSafeRegistry
     */
    function _deployHoprChannels() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.channelsContractAddress)
        ) {
            // deploy channels contract
            uint256 noticePeriodChannelClosure = currentEnvironmentType == EnvironmentType.LOCAL ? 15 : 5 * 60;
            currentNetworkDetail.addresses.channelsContractAddress = deployCode(
                "Channels.sol:HoprChannels",
                abi.encode(
                    currentNetworkDetail.addresses.tokenContractAddress,
                    noticePeriodChannelClosure,
                    currentNetworkDetail.addresses.nodeSafeRegistryAddress
                )
            );
            isHoprChannelsDeployed = true;
        }
    }

    /**
     * @dev deploy network registry proxy.
     * In development, dummy is used
     */
    function _deployNRProxy(address deployerAddress) internal {
        if (currentEnvironmentType == EnvironmentType.LOCAL) {
            // deploy DummyProxy in LOCAL environment
            currentNetworkDetail.addresses.networkRegistryProxyContractAddress = deployCode(
                "DummyProxyForNetworkRegistry.sol:HoprDummyProxyForNetworkRegistry", abi.encode(deployerAddress)
            );
            isHoprNetworkRegistryDeployed = true;
        } else if (!isValidAddress(currentNetworkDetail.addresses.networkRegistryProxyContractAddress)) {
            // deploy StakingProxy in other environment types, if no proxy contract is given.
            // temporarily grant default admin role to the deployer wallet
            currentNetworkDetail.addresses.networkRegistryProxyContractAddress = deployCode(
                "SafeProxyForNetworkRegistry.sol:HoprSafeProxyForNetworkRegistry",
                abi.encode(
                    deployerAddress,
                    owner,
                    0, // disable self-registry
                    block.number, // latest block number
                    currentNetworkDetail.addresses.tokenContractAddress,
                    currentNetworkDetail.addresses.nodeSafeRegistryAddress
                )
            );

            // swap owner and grant manager role to more wallets
            _helperSwapOwnerGrantManager(
                currentNetworkDetail.addresses.networkRegistryContractAddress, deployerAddress, owner
            );
            // flag isHoprNetworkRegistryDeployed
            isHoprNetworkRegistryDeployed = true;
        }
    }

    /**
     * @dev deploy network registry
     * in development environment, it's disabled
     */
    function _deployNetworkRegistry(address deployerAddress) internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.networkRegistryContractAddress)
        ) {
            // deploy NetworkRegistry contract
            // temporarily grant default admin role to the deployer wallet
            currentNetworkDetail.addresses.networkRegistryContractAddress = deployCode(
                "NetworkRegistry.sol:HoprNetworkRegistry",
                abi.encode(currentNetworkDetail.addresses.networkRegistryProxyContractAddress, deployerAddress, owner)
            );
            // swap owner and grant manager role to more wallets
            _helperSwapOwnerGrantManager(
                currentNetworkDetail.addresses.networkRegistryContractAddress, deployerAddress, owner
            );

            // NetworkRegistry should be enabled (default behavior) in staging/production, and disabled in development
            if (currentEnvironmentType == EnvironmentType.LOCAL) {
                (bool successDisableRegistry,) = currentNetworkDetail.addresses.networkRegistryContractAddress.call(
                    abi.encodeWithSignature("disableRegistry()")
                );
                if (!successDisableRegistry) {
                    emit log_string("Cannot disableRegistry");
                }
            }
        }
    }

    /**
     * @dev deploy ticket price oracle
     */
    function _deployHoprTicketPriceOracle(address deployerAddress, uint256 price) internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.ticketPriceOracleContractAddress)
        ) {
            // deploy contract
            currentNetworkDetail.addresses.ticketPriceOracleContractAddress =
                deployCode("TicketPriceOracle.sol:HoprTicketPriceOracle", abi.encode(deployerAddress, price));
        }
    }

    /**
     * @dev deploy Announcments smart contract and register NodeSafeRegistry
     */
    function _deployHoprAnnouncements() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.addresses.announcements)
        ) {
            // deploy HoprAnnouncements contract and register with current NodeSafeRegistry
            currentNetworkDetail.addresses.announcements = deployCode(
                "Announcements.sol:HoprAnnouncements",
                abi.encode(currentNetworkDetail.addresses.nodeSafeRegistryAddress)
            );
        }
    }

    /**
     * @dev helper function to
     * - grant manager role to manager addresses
     * - grant default admin role to the new owner
     * - renounce default admin role from the current caller
     * @param contractAddress address that has access control
     * @param caller caller address
     * @param newOwner new owner of the contract
     */
    function _helperSwapOwnerGrantManager(address contractAddress, address caller, address newOwner) internal {
        // grant default admin role to the actual owner
        (bool successGrantDefaultAdminRole,) =
            contractAddress.call(abi.encodeWithSignature("grantRole(bytes32,address)", DEFAULT_ADMIN_ROLE, newOwner));
        if (!successGrantDefaultAdminRole) {
            emit log_string("Cannot grant DEFAULT_ADMIN_ROLE role on ");
        }
        // grant manager roles to more accounts
        for (uint256 i = 0; i < PRODUCT_TEAM_MANAGER_ADDRESSES.length; i++) {
            (bool successGrantManagerRole,) = contractAddress.call(
                abi.encodeWithSignature("grantRole(bytes32,address)", MANAGER_ROLE, PRODUCT_TEAM_MANAGER_ADDRESSES[i])
            );
        }
        if (!successGrantDefaultAdminRole) {
            emit log_string("Cannot grant MANAGER_ROLE role on ");
        }
        // renounce the default admin role
        (bool successRenounceDefaultAdminRole,) =
            contractAddress.call(abi.encodeWithSignature("renounceRole(bytes32,address)", DEFAULT_ADMIN_ROLE, caller));
        if (!successRenounceDefaultAdminRole) {
            emit log_string("Cannot renounce DEFAULT_ADMIN_ROLE role on ");
        }
    }
}
