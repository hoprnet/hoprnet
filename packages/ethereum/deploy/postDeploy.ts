import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

// once contracts have been deployed, we run 'postDeploy' and try to verify on etherscan
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  if (hre.network.name === 'hardhat') return
  await hre.run('postDeploy')

  // try to verify
  if (hre.network.tags.etherscan) {
    await hre.run('etherscan-verify')
  }
}

main.runAtTheEnd = true

export default main
