import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { ERC1820_REGISTRY_ADDRESS, ERC1820_REGISTRY_DEPLOY_TX } from '@openzeppelin/test-helpers/src/data'

// Read https://eips.ethereum.org/EIPS/eip-1820 for more information as to how the ERC1820 registry is deployed to
// ensure its address is the same on all chains.
// This is very close to what OZ does: https://github.com/OpenZeppelin/openzeppelin-test-helpers/blob/330fb1c33ca87790d86ac7730cfb8a42a3bc0805/src/singletons.js#L15
// we can't use that utility since it doesn't work in public networks
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { web3, Web3, getNamedAccounts } = hre
  const { toWei } = Web3.utils
  const { deployer } = await getNamedAccounts()

  // check if it already exists
  if ((await web3.eth.getCode(ERC1820_REGISTRY_ADDRESS)).length > '0x'.length) {
    console.log('ERC1820 registry already exists')
    return
  }

  // 0.08 ether is needed to deploy the registry, and those funds need to be transferred to the account that will deploy
  // the contract.
  await web3.eth.sendTransaction({
    from: deployer,
    to: '0xa990077c3205cbDf861e17Fa532eeB069cE9fF96',
    value: toWei('0.08', 'ether')
  })

  // deploy
  await web3.eth.sendSignedTransaction(ERC1820_REGISTRY_DEPLOY_TX)

  console.log(`"ERC1820Registry" deployed at ${ERC1820_REGISTRY_ADDRESS}`)
}

export default main
