import type { DeployFunction } from 'hardhat-deploy/types'
import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import { HoprNetworkRegistry } from '../src/types'
import { MIN_STAKE } from '../utils/constants'

const DUMMY_PROXY = 'HoprDummyProxyForNetworkRegistry'
const STAKING_PROXY = 'HoprStakingProxyForNetworkRegistry'

// Deploy directly a HoprNetworkRegistry contract, using hardcoded staking contract.
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network, ethers, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const { deployer } = await getNamedAccounts()

  const stakeAddress = (await deployments.get('HoprStake')).address

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
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })

  console.log(
    `"HoprNetworkRegistryProxy" (${STAKING_PROXY ? 'StakingProxy.sol' : 'DummyProxy.sol'}) deployed at ${
      registryProxy.address
    }`
  )

  try {
    // if a NetworkRegistry contract instance exists, try to update with the latest proxy implementation
    const networkRegistry = await deployments.get('HoprNetworkRegistry')
    const registryContract = (await ethers.getContractFactory('HoprNetworkRegistry')).attach(
      networkRegistry.address
    ) as HoprNetworkRegistry
    const isImplementationDifferent =
      (await registryContract.requirementImplementation()).toLowerCase() !== registryProxy.address.toLowerCase()
    const isDeployerRegistryOwner = (await registryContract.owner()).toLowerCase() === deployer.toLowerCase()
    console.log(
      `Registry proxy implementation is ${isImplementationDifferent ? '' : 'not '}different; Deployer is ${
        isDeployerRegistryOwner ? '' : 'not '
      }owner.`
    )
    if (isImplementationDifferent && isDeployerRegistryOwner) {
      // update proxy in NR contract
      const updateTx = await registryContract.updateRequirementImplementation(registryProxy.address)

      // don't wait when using local hardhat because its using auto-mine
      if (!environment.match('hardhat')) {
        console.log(`Wait for minting tx on chain`)
        await ethers.provider.waitForTransaction(updateTx.hash, 2)
      }
    }
  } catch (error) {
    console.log('Cannot update proxy implementation.')
  }
}

main.dependencies = ['preDeploy', 'HoprStake']
main.tags = ['HoprNetworkRegistryProxy']

export default main
