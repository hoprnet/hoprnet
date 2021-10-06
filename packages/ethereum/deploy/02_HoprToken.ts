import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { HoprToken__factory } from '../types'
import { utils, constants } from 'ethers'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, network, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))

  const result = await deployments.deploy('HoprToken', {
    from: deployer.address,
    log: true
  })

  if (network.tags.testing || network.tags.development) {
    const hoprToken = new HoprToken__factory(deployer).attach(result.address)
    const MINTER_ROLE = await hoprToken.MINTER_ROLE()
    if (!(await hoprToken.hasRole(MINTER_ROLE, deployer.address))) {
      await hoprToken.grantRole(MINTER_ROLE, deployer.address)
      await hoprToken.mint(
        '0x2402da10A6172ED018AEEa22CA60EDe1F766655C',
        utils.parseEther('130000000'),
        constants.HashZero,
        constants.HashZero
      )
    }
  }
}

// this smart contract should not be redeployed
// in a live network
main.skip = async (env) => env.network.live

export default main
