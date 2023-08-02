// SPDX-License-Identifier: UNLICENSED
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

    function setUp() public override(ERC1820RegistryFixtureTest) {}

    function run() external {
        // 1. Network check
        // get environment of the script
        getNetwork();
        // read records of deployed files
        readCurrentNetwork();
        // Halt if ERC1820Registry has not been deployed.
        mustHaveErc1820Registry();
        emit log_string(string(abi.encodePacked("Deploying in ", currentNetworkId)));

        // 2. Get deployer internal key.
        // Set to default when it's in development environment (uint for 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80)
        uint256 deployerPrivateKey = currentEnvironmentType == EnvironmentType.LOCAL
            ? 77814517325470205911140941194401928579557062014761831930645393041380819009408
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
        // E.g. always in local environment, or should a new token contract be introduced in development/staging/production.
        _deployHoprTokenAndMintToAddress(deployerAddress, deployerAddress);

        // 3.5. HoprChannels Contract
        // Only deploy Channels contract when no deployed one is detected.
        // E.g. always in local environment, or should a new channel contract be introduced in development/staging/production per meta environment.
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.channelsContractAddress)
        ) {
            // deploy channels contract
            uint256 closure = currentEnvironmentType == EnvironmentType.LOCAL ? 15 : 5 * 60;
            currentNetworkDetail.channelsContractAddress = deployCode(
                "Channels.sol:HoprChannels",
                abi.encode(
                    currentNetworkDetail.tokenContractAddress, closure, currentNetworkDetail.nodeSafeRegistryAddress
                )
            );
            isHoprChannelsDeployed = true;
        }

        // 3.6. NetworkRegistryProxy Contract
        // Only deploy NetworkRegistryProxy contract when no deployed one is detected.
        // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in development/staging/production
        _deployNRProxy(deployerAddress);

        // 3.7. NetworkRegistry Contract
        // Only deploy NetworkRegistrycontract when no deployed one is detected.
        // E.g. Always in local environment, or should a new NetworkRegistryProxy contract be introduced in development/staging/production
        _deployNetworkRegistry(deployerAddress);

        // 3.8. TicketPriceOracle
        _deployHoprTicketPriceOracle(deployerAddress, 100);

        // 4. update indexerStartBlockNumber
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
     * @dev deploy node safe factory
     */
    function _deployHoprNodeStakeFactory() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.nodeStakeV2FactoryAddress)
        ) {
            // deploy HoprNodeStakeFactory contract
            currentNetworkDetail.nodeStakeV2FactoryAddress = deployCode("NodeStakeFactory.sol:HoprNodeStakeFactory");
        }
    }

    /**
     * @dev Deploy node management module
     */
    function _deployHoprNodeManagementModule() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.moduleImplementationAddress)
        ) {
            // deploy HoprNodeManagementModule contract
            currentNetworkDetail.moduleImplementationAddress =
                deployCode("NodeManagementModule.sol:HoprNodeManagementModule");
        }
    }

    /**
     * @dev Deploy node safe registry
     */
    function _deployHoprHoprNodeSafeRegistry() internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.nodeSafeRegistryAddress)
        ) {
            // deploy HoprNodeManagementModule contract
            currentNetworkDetail.nodeSafeRegistryAddress = deployCode("NodeSafeRegistry.sol:HoprNodeSafeRegistry");
        }
    }

    /**
     * @dev Deploy hopr token. Set a minter and mint some token to the deployer
     */
    function _deployHoprTokenAndMintToAddress(address deployerAddress, address recipient) internal {
        if (
            currentEnvironmentType == EnvironmentType.LOCAL
                || !isValidAddress(currentNetworkDetail.tokenContractAddress)
        ) {
            // deploy token contract
            currentNetworkDetail.tokenContractAddress = deployCode("HoprToken.sol");
            // grant deployer minter role
            (bool successGrantMinterRole,) = currentNetworkDetail.tokenContractAddress.call(
                abi.encodeWithSignature("grantRole(bytes32,address)", MINTER_ROLE, deployerAddress)
            );
            if (!successGrantMinterRole) {
                emit log_string("Cannot grantMinterRole");
            }
            // mint some tokens to the deployer
            (bool successMintTokens,) = currentNetworkDetail.tokenContractAddress.call(
                abi.encodeWithSignature(
                    "mint(address,uint256,bytes,bytes)", recipient, 130000000 ether, hex"00", hex"00"
                )
            );
            if (!successMintTokens) {
                emit log_string("Cannot mint tokens");
            }
        }
    }

    /**
     * @dev deploy network registry proxy.
     * In development, dummy is used
     */
    function _deployNRProxy(address deployerAddress) internal {
        if (currentEnvironmentType == EnvironmentType.LOCAL) {
            // deploy DummyProxy in LOCAL environment
            currentNetworkDetail.networkRegistryProxyContractAddress = deployCode(
                "DummyProxyForNetworkRegistry.sol:HoprDummyProxyForNetworkRegistry", abi.encode(deployerAddress)
            );
            isHoprNetworkRegistryDeployed = true;
        } else if (!isValidAddress(currentNetworkDetail.networkRegistryProxyContractAddress)) {
            // deploy StakingProxy in other environment types, if no proxy contract is given.
            currentNetworkDetail.networkRegistryProxyContractAddress = deployCode(
                "SafeProxyForNetworkRegistry.sol:HoprSafeProxyForNetworkRegistry",
                abi.encode(
                    COMM_MULTISIG_ADDRESS,
                    deployerAddress,
                    0, // disable self-registry
                    block.number, // latest block number
                    currentNetworkDetail.tokenContractAddress,
                    currentNetworkDetail.nodeSafeRegistryAddress
                )
            );
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
                || !isValidAddress(currentNetworkDetail.networkRegistryContractAddress)
        ) {
            // deploy NetworkRegistry contract
            currentNetworkDetail.networkRegistryContractAddress = deployCode(
                "NetworkRegistry.sol:HoprNetworkRegistry",
                abi.encode(
                    currentNetworkDetail.networkRegistryProxyContractAddress, COMM_MULTISIG_ADDRESS, deployerAddress
                )
            );
            // NetworkRegistry should be enabled (default behavior) in staging/production, and disabled in development
            if (currentEnvironmentType == EnvironmentType.LOCAL) {
                (bool successDisableRegistry,) = currentNetworkDetail.networkRegistryContractAddress.call(
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
                || !isValidAddress(currentNetworkDetail.ticketPriceOracleContractAddress)
        ) {
            // deploy contract
            currentNetworkDetail.ticketPriceOracleContractAddress =
                deployCode("TicketPriceOracle.sol:HoprTicketPriceOracle", abi.encode(deployerAddress, price));
        }
    }
}
