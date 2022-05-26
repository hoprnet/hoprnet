import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

const main = async function ({ ethers, deployments, getNamedAccounts, network }: HardhatRuntimeEnvironment) {
  const { deployer } = await getNamedAccounts()

  const inProd = network.name === 'xdai'

  // Local development environment uses HoprDummyProxyForNetworkRegistry. All the other network uses HoprStakingProxyForNetworkRegistry
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
    const isEnabled = await networkRegistry.enabled()
    if (isEnabled) {
      await networkRegistry.disableRegistry()
      console.log(`Disabled "HoprNetworkRegistry" in production.`)
    }
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
