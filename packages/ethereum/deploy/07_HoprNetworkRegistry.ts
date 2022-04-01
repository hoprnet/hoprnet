import type { HardhatRuntimeEnvironment } from 'hardhat/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

// Deploy directly a whitelist contract, using hardcoded staking contract.
const main = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const adminAddress =
    network.name == 'hardhat' ? deployer.address : environmentConfig['network_registry_admin_address']
  const registryProxy = environmentConfig['network_id'] == 'xdai' 
  ? await deployments.get('HoprStakingProxyForNetworkRegistry')
  : await deployments.get('HoprDummyProxyForNetworkRegistry')

  const networkRegistry = await deployments.deploy('HoprNetworkRegistry', {
    from: deployer.address,
    log: true,
    args: [registryProxy.address, adminAddress]
  })

  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistry.address}`)
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
