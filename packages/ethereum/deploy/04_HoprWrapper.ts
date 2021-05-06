import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { storeContract } from '../tasks/utils/contracts'

const XHOPR_ADDRESS = '0xD057604A14982FE8D88c5fC25Aac3267eA142a08'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, network, getNamedAccounts } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const hoprToken = await deployments.get('HoprToken')

  const result = await deployments.deploy('HoprWrapper', {
    from: deployer.address,
    args: [XHOPR_ADDRESS, hoprToken.address],
    log: true
  })
  await storeContract(network.name, network.tags, 'HoprWrapper', result.address, result.receipt.blockNumber)
}

// deploy this only on xdai
main.skip = async (env) => {
  return env.network.name !== 'xdai'
}

export default main
