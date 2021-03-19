import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../chain'
import { durations } from '@hoprnet/hopr-utils'

const closures: {
  [key in DeploymentTypes]: number
} = {
  local: durations.minutes(1),
  staging: durations.minutes(10),
  production: durations.minutes(60)
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()
  const deploymentType = Object.keys(network.tags).find((tag) => closures[tag])

  const hoprToken = await deployments.get('HoprToken')

  await deploy('HoprChannels', {
    from: deployer,
    args: [hoprToken.address, Math.floor(closures[deploymentType] ?? closures.local / 1e3)],
    log: true
  })
}

export default main
