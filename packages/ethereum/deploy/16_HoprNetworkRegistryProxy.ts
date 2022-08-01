import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import { MIN_STAKE } from '../utils/constants'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')
const DUMMY_PROXY = 'HoprDummyProxyForNetworkRegistry'
const STAKING_PROXY = 'HoprStakingProxyForNetworkRegistry'

// Deploy directly a HoprNetworkRegistry contract, using hardcoded staking contract.
const main = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network, environment } = hre
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const { deployer } = await getNamedAccounts()

  const stakeAddress =
    network.tags.testing || network.tags.development || network.tags.staging
      ? (await deployments.get('HoprStake')).address
      : environmentConfig['stake_contract_address']

  // Local development environment uses HoprDummyProxyForNetworkRegistry. All the other network uses HoprStakingProxyForNetworkRegistry
  const registryProxyName = network.name == 'hardhat' ? DUMMY_PROXY : STAKING_PROXY

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const registryProxy = await deployments.deploy('HoprNetworkRegistryProxy', {
    contract: registryProxyName,
    from: deployer,
    args: registryProxyName === STAKING_PROXY ? [stakeAddress, deployer, MIN_STAKE] : [deployer],
    ...deployOptions
  })

  console.log(`"HoprNetworkRegistryProxy" deployed at ${registryProxy.address}`)
}

main.dependencies = ['preDeploy', 'HoprStake']
main.tags = ['HoprNetworkRegistryProxy']

export default main
