import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../src/constants'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

  // CommonJS / ESM issue of `hardhat-core`
  const { durations, u8aToHex, pickVersion } = await import('@hoprnet/hopr-utils')

  const closures: {
    [key in DeploymentTypes]: number
  } = {
    testing: durations.seconds(15),
    development: durations.seconds(15),
    staging: durations.minutes(5),
    production: durations.minutes(5)
  }

  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const hoprToken = await deployments.get('HoprToken')

  // salt is used to ensure that a smart contract is re-deployed
  // on a new combination of environment and version `X.X.0`
  // this is necessary to ensure that all nodes that have announced
  // in HoprChannels can reach each other
  const version = pickVersion(require('../package.json').version)
  const salt = `${hre.environment}@${version}`

  const deploymentType = Object.keys(network.tags).find((tag) => closures[tag])
  const closure = Math.floor((closures[deploymentType] ?? closures.testing) / 1e3)

  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const result = await deployments.deterministic('HoprChannels', {
    from: deployer.address,
    args: [hoprToken.address, closure],
    salt: u8aToHex(new TextEncoder().encode(salt)),
    ...deployOptions
  })

  await result.deploy()
}

main.dependencies = ['preDeploy', 'HoprToken']
main.tags = ['HoprChannels']

export default main
