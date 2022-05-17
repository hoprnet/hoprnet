import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

const main = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const inProd = environmentConfig['network_id'] == 'xdai'

  const adminAddress =
    network.name == 'hardhat' ? deployer.address : environmentConfig['network_registry_admin_address']
  const registryProxy = inProd
    ? await deployments.get('HoprStakingProxyForNetworkRegistry')
    : await deployments.get('HoprDummyProxyForNetworkRegistry')

  const networkRegistryContract = await deployments.deploy('HoprNetworkRegistry', {
    from: deployer.address,
    log: true,
    args: [registryProxy.address, adminAddress]
  })

  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistryContract.address}`)

  if (!inProd) {
    const networkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry')).attach(
      networkRegistryContract.address
    ) as HoprNetworkRegistry
    await networkRegistry.disableRegistry()
    console.log(`Disabled "HoprNetworkRegistry"`)
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
