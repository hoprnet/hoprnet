import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { HoprToken__factory } from '../types'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, network, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const result = await deployments.deploy('HoprToken', {
    from: deployer.address,
    log: true
  })

  if (network.name === 'localhost') {
    const hoprToken = new HoprToken__factory(deployer).attach(result.address)
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()
    if (!(await hoprToken.hasRole(MINTER_ROLE, deployer.address))) {
      await hoprToken.grantRole(MINTER_ROLE, deployer.address)
    }
  }
}

export default main
