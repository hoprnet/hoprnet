import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

const main = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre
  const { deployer } = await getNamedAccounts()

  const inProd = network.name === 'xdai'

  // Local development environment uses HoprDummyProxyForNetworkRegistry. All the other network uses HoprStakingProxyForNetworkRegistry
  const registryProxy = await deployments.get('HoprNetworkRegistryProxy')

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const networkRegistryContract = await deployments.deploy('HoprNetworkRegistry', {
    from: deployer,
    args: [registryProxy.address, deployer],
    ...deployOptions
  })
  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistryContract.address}`)

  const networkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry')).attach(
    networkRegistryContract.address
  ) as HoprNetworkRegistry

  if (!inProd) {
    const isEnabled = await networkRegistry.enabled()
    if (isEnabled) {
      const disableTx = await networkRegistry.disableRegistry()

      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(disableTx.hash, 2)
      }

      console.log(`Disabled "HoprNetworkRegistry" in production.`)
    }
  }
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
