import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const XHOPR_ADDRESS = '0xD057604A14982FE8D88c5fC25Aac3267eA142a08'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()
  const hoprToken = await deployments.get('HoprToken')

  await deploy('HoprWrapper', {
    from: deployer,
    args: [XHOPR_ADDRESS, hoprToken.address],
    log: true
  })
}

// deploy this only on xdai
main.skip = async (env) => {
  return env.network.name !== 'xdai'
}

export default main
