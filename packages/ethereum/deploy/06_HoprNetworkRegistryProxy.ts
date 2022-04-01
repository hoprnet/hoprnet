import type { HardhatRuntimeEnvironment } from 'hardhat/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')
const MIN_STAKE = 0

// Deploy directly a whitelist contract, using hardcoded staking contract.
const main = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const adminAddress =
    network.name == 'hardhat' ? deployer.address : environmentConfig['network_registry_admin_address']

  // deploy different contracts depending on the environment
  const registryProxy =
    environmentConfig['network_id'] == 'xdai'
      ? // deploy "HoprStakingProxyForNetworkRegistry" contract for releases on Gnosis chain (xDai)
        await deployments.deploy('HoprStakingProxyForNetworkRegistry', {
          from: deployer.address,
          log: true,
          args: [environmentConfig['stake_v2_contract_address'], adminAddress, MIN_STAKE]
        })
      : // deploy "HoprDummyProxyForNetworkRegistry" contract for the rest
        await deployments.deploy('HoprDummyProxyForNetworkRegistry', {
          from: deployer.address,
          log: true,
          args: [adminAddress]
        })

  console.log(`"HoprNetworkRegistryProxy" deployed at ${registryProxy.address}`)
}

main.dependencies = ['preDeploy']
main.tags = ['HoprNetworkRegistryProxy']

export default main
