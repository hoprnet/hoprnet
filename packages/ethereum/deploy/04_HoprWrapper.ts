import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const XHOPR_ADDRESS = '0xD057604A14982FE8D88c5fC25Aac3267eA142a08'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const hoprToken = await deployments.get('HoprToken')

  await deployments.deploy('HoprWrapper', {
    from: deployer.address,
    args: [XHOPR_ADDRESS, hoprToken.address],
    log: true
  })
}

// this smart contract should not be redeployed
// in a live network
main.skip = async (env) => {
  return env.network.live || env.network.name !== 'xdai'
}

export default main
