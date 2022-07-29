import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  await deployments.deploy('HoprBoost', {
    from: deployer.address,
    args: [deployer.address, ''],
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })
}

main.tags = ['HoprBoost']
main.dependencies = ['preDeploy', 'xHoprMock']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main
