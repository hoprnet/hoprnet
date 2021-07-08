import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const hoprDistributor = await deployments.get('HoprDistributor')

  await deployments.deploy('HoprRegistry', {
    from: deployer.address,
    args: [hoprDistributor.address],
    log: true
  })
}

export default main
