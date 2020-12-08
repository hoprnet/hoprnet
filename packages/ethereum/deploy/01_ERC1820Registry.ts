import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { singletons } from '@openzeppelin/test-helpers'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { getNamedAccounts } = hre
  const { deployer } = await getNamedAccounts()

  console.log('Deploying ERC1820Registry')
  const ERC1820Registry = await singletons.ERC1820Registry(deployer)
  console.log(`Deployed or Found ERC1820Registry: ${ERC1820Registry.address}`)
}

export default main
