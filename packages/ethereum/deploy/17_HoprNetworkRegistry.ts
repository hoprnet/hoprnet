import type { HardhatRuntimeEnvironment } from 'hardhat/types'

const main = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre

  const { deployer, admin } = await getNamedAccounts()

  // FIXME: All the network uses HoprStakingProxyForNetworkRegistry
  // const registryProxy =
  //   environmentConfig['network_id'] == 'xdai'
  //     ? await deployments.get('HoprStakingProxyForNetworkRegistry')
  //     : await deployments.get('HoprDummyProxyForNetworkRegistry')
  const registryProxy = await deployments.get('HoprNetworkRegistryProxy')

  const networkRegistry = await deployments.deploy('HoprNetworkRegistry', {
    from: deployer,
    log: true,
    args: [registryProxy.address, admin]
  })

  console.log(`"HoprNetworkRegistry" deployed at ${networkRegistry.address}`)
}

main.dependencies = ['preDeploy', 'HoprNetworkRegistryProxy']
main.tags = ['HoprNetworkRegistry']

export default main
