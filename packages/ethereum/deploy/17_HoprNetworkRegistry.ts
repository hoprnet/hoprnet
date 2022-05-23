import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')


const main = async function ({ ethers, deployments, getNamedAccounts, environment }: HardhatRuntimeEnvironment) {
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]

  const { deployer, admin } = await getNamedAccounts()

  const inProd = environmentConfig['network_id'] == 'xdai';

  // FIXME: All the network uses HoprStakingProxyForNetworkRegistry
  // const registryProxy =
  //   inProd
  //     ? await deployments.get('HoprStakingProxyForNetworkRegistry')
  //     : await deployments.get('HoprDummyProxyForNetworkRegistry')
  const registryProxy = await deployments.get('HoprNetworkRegistryProxy')

  const networkRegistryContract = await deployments.deploy('HoprNetworkRegistry', {
    from: deployer,
    log: true,
    args: [registryProxy.address, deployer]
  })

  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistryContract.address}`)

  const networkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry')).attach(
    networkRegistryContract.address
  ) as HoprNetworkRegistry

  if (!inProd) {
    await networkRegistry.disableRegistry()
    console.log(`Disabled "HoprNetworkRegistry" in production.`)
  }

  if (deployer !== admin) {
    await networkRegistry.transferOwnership(admin);
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
