import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../constants'
import { durations, u8aToHex } from '@hoprnet/hopr-utils'

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

  let salt: string
  if (hre.environment === 'default') {
    salt = require('../package.json').version
  } else {
    salt = hre.environment
  }

  const result = await deployments.deterministic('HoprChannels', {
    from: deployer.address,
    args: [hoprToken.address, Math.floor((closures[deploymentType] ?? closures.testing) / 1e3)],
    salt: u8aToHex(new TextEncoder().encode(salt)),
    log: true
  })

  await result.deploy()
}

export default main
