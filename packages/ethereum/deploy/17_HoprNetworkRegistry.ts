import type { DeployFunction } from 'hardhat-deploy/types'
import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { HoprNetworkRegistry } from '../src/types'

/**
 * This Network Registry contract should not be redeployed, even when the proxy is deployed at a new address
 */
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const { deployer } = await getNamedAccounts()

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
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })
  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistryContract.address}`)

  const networkRegistry = (await ethers.getContractFactory('HoprNetworkRegistry')).attach(
    networkRegistryContract.address
  ) as HoprNetworkRegistry

  const isEnabled = await networkRegistry.enabled()

  // when in production, network registry should be enabled
  // when in staging, network registry should be enabled.
  // for other environment, network registry should be disabled
  if (network.tags.staging || network.tags.production) {
    if (!isEnabled) {
      await networkRegistry.enableRegistry()
      console.log(`Enabled "HoprNetworkRegistry" in staging.`)
    }
  } else {
    if (isEnabled) {
      const disableTx = await networkRegistry.disableRegistry()

      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        await ethers.provider.waitForTransaction(disableTx.hash, 2)
      }

      console.log(`Disabled "HoprNetworkRegistry" in production.`)
    }
  }
  console.log(
    `NetworkRegistry on ${network.name} network with tags ${Object.keys(network.tags)} is ${
      (await networkRegistry.enabled()) ? 'enabled' : 'disabled'
    }.`
  )
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main
