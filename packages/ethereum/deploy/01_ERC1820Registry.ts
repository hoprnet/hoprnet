import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { ERC1820_REGISTRY_ADDRESS, ERC1820_REGISTRY_DEPLOY_TX } from '@openzeppelin/test-helpers/src/data'

// Read https://eips.ethereum.org/EIPS/eip-1820 for more information as to how the ERC1820 registry is deployed to
// ensure its address is the same on all chains.
// This is very close to what OZ does: https://github.com/OpenZeppelin/openzeppelin-test-helpers/blob/330fb1c33ca87790d86ac7730cfb8a42a3bc0805/src/singletons.js#L15
// we can't use that utility since it doesn't work in public networks
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  // check if it already exists
  if ((await ethers.provider.getCode(ERC1820_REGISTRY_ADDRESS)).length > '0x'.length) {
    console.log('ERC1820 registry already exists')
    return
  }

  // 0.08 ether is needed to deploy the registry, and those funds need to be transferred to the account that will deploy
  // the contract.
  await deployer.sendTransaction({
    to: '0xa990077c3205cbDf861e17Fa532eeB069cE9fF96',
    value: ethers.utils.parseEther('0.08')
  })

  // deploy
  await ethers.provider.sendTransaction(ERC1820_REGISTRY_DEPLOY_TX)

  console.log(`"ERC1820Registry" deployed at ${ERC1820_REGISTRY_ADDRESS}`)
}

export default main
