import type { HardhatRuntimeEnvironment } from 'hardhat/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')
const MIN_STAKE = 0
const DUMMY_PROXY = 'HoprDummyProxyForNetworkRegistry'
const STAKING_PROXY = 'HoprStakingProxyForNetworkRegistry'

// Deploy directly a HoprNetworkRegistry contract, using hardcoded staking contract.
const main = async function ({ deployments, getNamedAccounts, network, environment }: HardhatRuntimeEnvironment) {
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const { deployer } = await getNamedAccounts()

  const stakeAddress =
    network.tags.testing || network.tags.development
      ? (await deployments.get('HoprStake')).address
      : environmentConfig['stake_contract_address']

  // Local development environment uses HoprDummyProxyForNetworkRegistry. All the other network uses HoprStakingProxyForNetworkRegistry
  const registryProxyName = network.name == 'hardhat' ? DUMMY_PROXY : STAKING_PROXY

  const registryProxy = await deployments.deploy('HoprNetworkRegistryProxy', {
    contract: registryProxyName,
    from: deployer,
    log: true,
    args: registryProxyName === STAKING_PROXY ? [stakeAddress, deployer, MIN_STAKE] : [deployer]
  })

  console.log(`"HoprNetworkRegistryProxy" deployed at ${registryProxy.address}`)
}

main.dependencies = ['preDeploy', 'HoprStake']
main.tags = ['HoprNetworkRegistryProxy']

export default main
