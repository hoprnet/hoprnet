import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { durations } from '@hoprnet/hopr-utils'

const SECS_CLOSURE = Math.floor(durations.minutes(1) / 1e3)

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()

  const hoprToken = await deployments.get('HoprToken')

  await deploy('HoprChannels', {
    from: deployer,
    args: [hoprToken.address, SECS_CLOSURE],
    log: true
  })
}

export default main
