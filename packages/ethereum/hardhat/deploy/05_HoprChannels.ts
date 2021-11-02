import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../../src/constants'
import { durations, u8aToHex, pickVersion } from '@hoprnet/hopr-utils'

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

  // salt is used to ensure that a smart contract is re-deployed
  // on new environments OR a changed version `X.X.0`
  // this is necessary to ensure that all nodes that have announced
  // in HoprChannels can reach each other
  let salt: string
  if (hre.environment === 'default') {
    // version bumping happens AFTER deployments are run
    // this means that the salt used here always used an outdated version
    // see https://github.com/hoprnet/hoprnet/issues/2635
    salt = pickVersion(require('../package.json').version)
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
