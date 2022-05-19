import type { HardhatRuntimeEnvironment } from 'hardhat/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')
const MIN_STAKE = 0

// Deploy directly a HoprNetworkRegistry contract, using hardcoded staking contract.
const main = async function ({
  ethers,
  deployments,
  getNamedAccounts,
  network,
  environment
}: HardhatRuntimeEnvironment) {
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const { deployer, admin } = await getNamedAccounts()

  const stakeAddress =
    network.tags.testing || network.tags.development
      ? (await deployments.get('HoprStake')).address
      : environmentConfig['stake_contract_address']

  // FIXME: All the network uses HoprStakingProxyForNetworkRegistry
  // // deploy different contracts depending on the environment
  // const registryProxy =
  //   !!network.tags.production || !!network.tags.staging // environmentConfig['network_id'] == 'xdai' || environmentConfig['network_id'] == 'goerli'
  //     ? // deploy "HoprStakingProxyForNetworkRegistry" contract for releases on Gnosis chain (xDai)
  //       await deployments.deploy('HoprNetworkRegistryProxy', {
  //         contract: "HoprStakingProxyForNetworkRegistry",
  //         from: deployer.address,
  //         log: true,
  //         args: [environmentConfig['stake_contract_address'], admin, MIN_STAKE]
  //       })
  //     : // deploy "HoprDummyProxyForNetworkRegistry" contract for the rest
  //       await deployments.deploy('HoprNetworkRegistryProxy', {
  //         contract: "HoprDummyProxyForNetworkRegistry",
  //         from: deployer.address,
  //         log: true,
  //         args: [admin]
  //       })

  // deploy different contracts depending on the environment
  await deployments.deploy('HoprNetworkRegistryProxy', {
    contract: 'HoprStakingProxyForNetworkRegistry',
    from: deployer,
    log: true,
    args: [stakeAddress, admin, MIN_STAKE]
  })
}

main.dependencies = ['preDeploy']
main.tags = ['HoprNetworkRegistryProxy']

export default main
