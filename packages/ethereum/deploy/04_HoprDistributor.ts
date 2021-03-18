import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'

const SECS_DISTRIBUTOR = Math.floor(durations.days(1) / 1e3)
const MAX_MINT_AMOUNT = Web3.utils.toWei('100000000', 'ether')

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()

  const hoprToken = await deployments.get('HoprToken')

  await deploy('HoprDistributor', {
    from: deployer,
    args: [hoprToken.address, SECS_DISTRIBUTOR, MAX_MINT_AMOUNT],
    log: true
  })
}

export default main
