import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const HoprToken = artifacts.require('HoprToken')

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()

  const result = await deploy('HoprToken', {
    from: deployer,
    log: true
  })

  if (network.name === 'localhost') {
    const hoprToken = await HoprToken.at(result.address)
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()
    if (!(await hoprToken.hasRole(MINTER_ROLE, deployer))) {
      await hoprToken.grantRole(MINTER_ROLE, deployer)
    }
  }
}

export default main
