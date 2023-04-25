pragma solidity >=0.8.0 <0.9.0;

import 'forge-std/Script.sol';
import 'forge-std/StdJson.sol';

/**
 * Get environment_type from the envrionment variable `FOUNDRY_PROFILE`
 * Get network_name string from the envrionment variable "NETWORK_NAME"
 */
contract NetworkConfig is Script {
  using stdJson for string;

  enum EnvironmentType {
    DEVELOPMENT,
    STAGING,
    PRODUCTION
  }

  struct NetworkDetail {
    EnvironmentType environmentType;
    uint256 stakeSeason;
    address hoprTokenContractAddress;
    address hoprChannelsContractAddress;
    address xhoprTokenContractAddress;
    address hoprBoostContractAddress;
    address stakeContractAddress;
    address networkRegistryContractAddress;
    address networkRegistryProxyContractAddress;
    uint256 indexerStartBlockNumber;
  }

  // Deployed contract addresses
  // address constant PROD_WXHOPR_TOKEN_CONTRACT_ADDRESS = 0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1; // TODO: this contract is not necessarily the "HoprToken" contract used in releases
  address constant PROD_XHOPR_TOKEN_CONTRACT_ADDRESS = 0xD057604A14982FE8D88c5fC25Aac3267eA142a08;
  address constant PROD_HOPR_BOOST_CONTRACT_ADDRESS = 0x43d13D7B83607F14335cF2cB75E87dA369D056c7;
  uint256 constant NETWORK_REGISTRY_NFT_INDEX = 26;
  string constant NETWORK_REGISTRY_TYPE_NAME = 'Network_registry';
  string constant NETWORK_REGISTRY_RANK1_NAME = 'developer';
  string constant NETWORK_REGISTRY_RANK2_NAME = 'community';
  string constant DUMMY_TYPE_PREFIX = 'Dummy_';
  bytes32 constant NETWORK_REGISTRY_TYPE_HASH = keccak256(bytes(NETWORK_REGISTRY_TYPE_NAME));
  bytes32 constant MINTER_ROLE = keccak256('MINTER_ROLE');
  address constant DEV_BANK_ADDRESS = 0x2402da10A6172ED018AEEa22CA60EDe1F766655C;

  string public currentNetworkName;
  EnvironmentType public currentEnvironmentType;
  NetworkDetail public currentNetworkDetail;

  string public pathToDeploymentFile = string(abi.encodePacked(vm.projectRoot(), '/contracts-addresses.json'));

  function getNetwork() public {
    // get envirionment of the script
    string memory profile = vm.envString('FOUNDRY_PROFILE');
    currentNetworkName = vm.envString('NETWORK_NAME');
    currentEnvironmentType = parseEnvironmentTypeFromString(profile);
  }

  function readNetwork(string memory _networkName) internal view returns (NetworkDetail memory networkDetail) {
    string memory json = vm.readFile(pathToDeploymentFile);
    bytes memory levelToNetworkConfig = abi.encodePacked('.networks.', _networkName);

    // read all the contract addresses from contracts-addresses.json. This way ensures that the order of attributes does not affect parsing
    EnvironmentType envType = parseEnvironmentTypeFromString(
      json.readString(string(abi.encodePacked(levelToNetworkConfig, '.environment_type')))
    );
    uint256 stakeSeasonNum = json.readUint(string(abi.encodePacked(levelToNetworkConfig, '.stake_season')));
    uint256 indexerStartBlkNum = json.readUint(
      string(abi.encodePacked(levelToNetworkConfig, '.indexer_start_block_number'))
    );
    address tokenAddr = json.readAddress(string(abi.encodePacked(levelToNetworkConfig, '.token_contract_address')));
    address channelAddr = json.readAddress(
      string(abi.encodePacked(levelToNetworkConfig, '.channels_contract_address'))
    );
    address xhoprAddr = json.readAddress(string(abi.encodePacked(levelToNetworkConfig, '.xhopr_contract_address')));
    address boostAddr = json.readAddress(string(abi.encodePacked(levelToNetworkConfig, '.boost_contract_address')));
    address stakeAddr = json.readAddress(string(abi.encodePacked(levelToNetworkConfig, '.stake_contract_address')));
    address networkRegistryProxyAddr = json.readAddress(
      string(abi.encodePacked(levelToNetworkConfig, '.network_registry_proxy_contract_address'))
    );
    address networkRegistryAddr = json.readAddress(
      string(abi.encodePacked(levelToNetworkConfig, '.network_registry_contract_address'))
    );

    networkDetail = NetworkDetail({
      environmentType: envType,
      stakeSeason: stakeSeasonNum,
      hoprTokenContractAddress: tokenAddr,
      hoprChannelsContractAddress: channelAddr,
      xhoprTokenContractAddress: xhoprAddr,
      hoprBoostContractAddress: boostAddr,
      stakeContractAddress: stakeAddr,
      networkRegistryContractAddress: networkRegistryAddr,
      networkRegistryProxyContractAddress: networkRegistryProxyAddr,
      indexerStartBlockNumber: indexerStartBlkNum
    });
  }

  function readCurrentNetwork() internal {
    currentNetworkDetail = readNetwork(currentNetworkName);
  }

  function writeNetwork(string memory _networkName, NetworkDetail memory networkDetail) internal {
    string memory parsedNewEnvDetail = parseNetworkDetailToString(networkDetail);

    // write parsedNewEnvDetail to corresponding key
    string memory configKey = string(abi.encodePacked('.networks.', _networkName));

    // write to file;
    vm.writeJson(parsedNewEnvDetail, pathToDeploymentFile, configKey);
  }

  function writeCurrentNetwork() internal {
    // if currentNetworkName is anvil-localhost, update both `anvil-localhost` and `anvil-localhost2`
    if (keccak256(bytes(currentNetworkName)) == keccak256(bytes('anvil-localhost'))) {
      writeNetwork('anvil-localhost2', currentNetworkDetail);
    }
    writeNetwork(currentNetworkName, currentNetworkDetail);
  }

  // FIXME: remove this temporary method
  function displayNetworkDetail(string memory filePath, NetworkDetail memory networkDetail) internal {
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked('"environment_type": "', parseEnvironmentTypeToString(networkDetail.environmentType), '",')
      )
    );
    vm.writeLine(filePath, string(abi.encodePacked('"stake_season": ', vm.toString(networkDetail.stakeSeason), ',')));
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"indexer_start_block_umber": ', vm.toString(networkDetail.indexerStartBlockNumber), ','))
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"token_contract_address": "', vm.toString(networkDetail.hoprTokenContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked('"channels_contract_address": "', vm.toString(networkDetail.hoprChannelsContractAddress), '",')
      )
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked('"xhopr_contract_address": "', vm.toString(networkDetail.xhoprTokenContractAddress), '",')
      )
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"boost_contract_address": "', vm.toString(networkDetail.hoprBoostContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"stake_contract_address": "', vm.toString(networkDetail.stakeContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked(
          '"network_registry_proxy_contract_address": "',
          vm.toString(networkDetail.networkRegistryProxyContractAddress),
          '",'
        )
      )
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked(
          '"network_registry_contract_address": "',
          vm.toString(networkDetail.networkRegistryContractAddress),
          '"'
        )
      )
    );
  }

  // FIXME: remove this temporary method
  function displayCurrentNetworkDetail() internal {
    displayNetworkDetail('test.txt', currentNetworkDetail);
  }

  function isValidAddress(address addr) public pure returns (bool) {
    return addr == address(32) || addr == address(0) ? false : true;
  }

  function parseEnvironmentTypeFromString(string memory environmentType) public pure returns (EnvironmentType) {
    if (keccak256(bytes(environmentType)) == keccak256(bytes('production'))) {
      return EnvironmentType.PRODUCTION;
    } else if (keccak256(bytes(environmentType)) == keccak256(bytes('staging'))) {
      return EnvironmentType.STAGING;
    } else {
      return EnvironmentType.DEVELOPMENT;
    }
  }

  function parseEnvironmentTypeToString(EnvironmentType environmentType) public pure returns (string memory) {
    if (environmentType == EnvironmentType.PRODUCTION) {
      return 'production';
    } else if (environmentType == EnvironmentType.STAGING) {
      return 'staging';
    } else {
      return 'development';
    }
  }

  function parseNetworkDetailToString(NetworkDetail memory networkDetail) internal returns (string memory) {
    string memory json = 'config';
    json.serialize('environment_type', parseEnvironmentTypeToString(networkDetail.environmentType));
    json.serialize('stake_season', networkDetail.stakeSeason);
    json.serialize('indexer_start_block_number', networkDetail.indexerStartBlockNumber);
    json.serialize('token_contract_address', networkDetail.hoprTokenContractAddress);
    json.serialize('channels_contract_address', networkDetail.hoprChannelsContractAddress);
    json.serialize('xhopr_contract_address', networkDetail.xhoprTokenContractAddress);
    json.serialize('boost_contract_address', networkDetail.hoprBoostContractAddress);
    json.serialize('stake_contract_address', networkDetail.stakeContractAddress);
    json.serialize('network_registry_proxy_contract_address', networkDetail.networkRegistryProxyContractAddress);
    json = json.serialize('network_registry_contract_address', networkDetail.networkRegistryContractAddress);
    return json;
  }
}
