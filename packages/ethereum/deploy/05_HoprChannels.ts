import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../constants'
import { durations } from '@hoprnet/hopr-utils'

const closures: {
  [key in DeploymentTypes]: number
} = {
  testing: durations.minutes(1),
  development: durations.minutes(1),
  staging: durations.minutes(5),
  production: durations.minutes(5)
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const deploymentType = Object.keys(network.tags).find((tag) => closures[tag])

  const hoprToken = await deployments.get('HoprToken')

  await deployments.deploy('HoprChannels', {
    from: deployer.address,
    args: [hoprToken.address, Math.floor((closures[deploymentType] ?? closures.testing) / 1e3)],
    log: true
  })
}

export default main
