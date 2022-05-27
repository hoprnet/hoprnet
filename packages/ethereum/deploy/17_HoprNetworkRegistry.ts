import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

const main = async function ({ ethers, deployments, getNamedAccounts, network }: HardhatRuntimeEnvironment) {
  const { deployer } = await getNamedAccounts()

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

  const isEnabled = await networkRegistry.enabled()

  // when in production, network registry should be disabled // FIXME: for the moment, until NR is fully tested in staging
  // when in staging, network registry should be enabled.
  // for other environment, network registry should be disabled
  if (network.tags.production) {
    if (isEnabled) {
      await networkRegistry.disableRegistry()
      console.log(`Disabled "HoprNetworkRegistry" in production.`)
    }
  } else if (network.tags.staging) {
    if (!isEnabled) {
      await networkRegistry.enableRegistry()
      console.log(`Enabled "HoprNetworkRegistry" in staging.`)
    }
  } else {
    if (isEnabled) {
      await networkRegistry.disableRegistry()
      console.log(`Disabled "HoprNetworkRegistry" in production.`)
    }
  }
  console.log(`NetworkRegistry on ${network.name} network with tags ${Object.keys(network.tags)} is ${(await networkRegistry.enabled())? "enabled": "disabled"}.`)
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
