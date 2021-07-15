import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import slimDeployments from '../tasks/slimDeployments'

// runs once deployment has finished
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  await slimDeployments(hre)

  // verify smart contract to etherscan
  if (hre.network.tags.etherscan) {
    await hre.run('etherscan-verify')
  }
}

main.runAtTheEnd = true

export default main
