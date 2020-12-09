import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

// once contracts have been deployed, we run 'postDeploy' task
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  await hre.run('postDeploy')
}

main.runAtTheEnd = true

export default main
