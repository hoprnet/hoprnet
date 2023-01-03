pragma solidity >=0.8.0 <0.9.0;

import 'forge-std/Script.sol';
import 'forge-std/StdJson.sol';

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

  string public currentEnvironmentName;
  EnvironmentType public currentEnvironmentType;
  EnvironmentDetail public currentEnvironmentDetail;

  string public pathToDeploymentFile = string(abi.encodePacked(vm.projectRoot(), '/contracts-addresses.json'));

  function getEnvironment() public {
    // get envirionment of the script
    string memory profile = vm.envString('FOUNDRY_PROFILE');
    currentEnvironmentName = vm.envString('ENVIRONMENT_NAME');
    currentEnvironmentType = parseEnvironmentTypeFromString(profile);
  }

  function readEnvironment(string memory _environmentName) internal view returns (EnvironmentDetail memory envDetail) {
    string memory json = vm.readFile(pathToDeploymentFile);
    bytes memory levelToEnvironmentConfig = abi.encodePacked('.environments.', _environmentName);

    // read all the contract addresses from contracts-addresses.json. This way ensures that the order of attributes does not affect parsing
    EnvironmentType envType = parseEnvironmentTypeFromString(
      json.readString(string(abi.encodePacked(levelToEnvironmentConfig, '.environment_type')))
    );
    uint256 stakeSeasonNum = json.readUint(string(abi.encodePacked(levelToEnvironmentConfig, '.stake_season')));
    uint256 indexerStartBlkNum = json.readUint(
      string(abi.encodePacked(levelToEnvironmentConfig, '.indexer_start_block_number'))
    );
    address tokenAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, '.token_contract_address')));
    address channelAddr = json.readAddress(
      string(abi.encodePacked(levelToEnvironmentConfig, '.channels_contract_address'))
    );
    address xhoprAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, '.xhopr_contract_address')));
    address boostAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, '.boost_contract_address')));
    address stakeAddr = json.readAddress(string(abi.encodePacked(levelToEnvironmentConfig, '.stake_contract_address')));
    address networkRegistryProxyAddr = json.readAddress(
      string(abi.encodePacked(levelToEnvironmentConfig, '.network_registry_proxy_contract_address'))
    );
    address networkRegistryAddr = json.readAddress(
      string(abi.encodePacked(levelToEnvironmentConfig, '.network_registry_contract_address'))
    );

    envDetail = EnvironmentDetail({
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

  function readCurrentEnvironment() internal {
    currentEnvironmentDetail = readEnvironment(currentEnvironmentName);
  }

  function writeEnvironment(string memory _environmentName, EnvironmentDetail memory envDetail) internal {
    string memory parsedNewEnvDetail = parseEnvironmentDetailToString(envDetail);

    // write parsedNewEnvDetail to corresponding key
    string memory configKey = string(abi.encodePacked('.environments.', _environmentName));

    // write to file;
    vm.writeJson(parsedNewEnvDetail, pathToDeploymentFile, configKey);
  }

  function writeCurrentEnvironment() internal {
    // if currentEnvironmentName is anvil-localhost, update both `anvil-localhost` and `anvil-localhost2`
    if (keccak256(bytes(currentEnvironmentName)) == keccak256(bytes('anvil-localhost'))) {
      writeEnvironment('anvil-localhost2', currentEnvironmentDetail);
    }
    writeEnvironment(currentEnvironmentName, currentEnvironmentDetail);
  }

  // FIXME: remove this temporary method
  function displayEnvironmentDetail(string memory filePath, EnvironmentDetail memory envDetail) internal {
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"environment_type": "', parseEnvironmentTypeToString(envDetail.environmentType), '",'))
    );
    vm.writeLine(filePath, string(abi.encodePacked('"stake_season": ', vm.toString(envDetail.stakeSeason), ',')));
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"indexer_start_block_umber": ', vm.toString(envDetail.indexerStartBlockNumber), ','))
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"token_contract_address": "', vm.toString(envDetail.hoprTokenContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked('"channels_contract_address": "', vm.toString(envDetail.hoprChannelsContractAddress), '",')
      )
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"xhopr_contract_address": "', vm.toString(envDetail.xhoprTokenContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"boost_contract_address": "', vm.toString(envDetail.hoprBoostContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(abi.encodePacked('"stake_contract_address": "', vm.toString(envDetail.stakeContractAddress), '",'))
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked(
          '"network_registry_proxy_contract_address": "',
          vm.toString(envDetail.networkRegistryProxyContractAddress),
          '",'
        )
      )
    );
    vm.writeLine(
      filePath,
      string(
        abi.encodePacked(
          '"network_registry_contract_address": "',
          vm.toString(envDetail.networkRegistryContractAddress),
          '"'
        )
      )
    );
  }

  // FIXME: remove this temporary method
  function displayCurrentEnvironmentDetail() internal {
    displayEnvironmentDetail('test.txt', currentEnvironmentDetail);
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

  function parseEnvironmentDetailToString(EnvironmentDetail memory envDetail) internal returns (string memory) {
    string memory json = 'config';
    json.serialize('environment_type', parseEnvironmentTypeToString(envDetail.environmentType));
    json.serialize('stake_season', envDetail.stakeSeason);
    json.serialize('indexer_start_block_number', envDetail.indexerStartBlockNumber);
    json.serialize('token_contract_address', envDetail.hoprTokenContractAddress);
    json.serialize('channels_contract_address', envDetail.hoprChannelsContractAddress);
    json.serialize('xhopr_contract_address', envDetail.xhoprTokenContractAddress);
    json.serialize('boost_contract_address', envDetail.hoprBoostContractAddress);
    json.serialize('stake_contract_address', envDetail.stakeContractAddress);
    json.serialize('network_registry_proxy_contract_address', envDetail.networkRegistryProxyContractAddress);
    json = json.serialize('network_registry_contract_address', envDetail.networkRegistryContractAddress);
    return json;
  }
}
