import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../src/constants'

const shortDuration = 15 * 1e3 // 15 seconds in ms
const longDuration = 5 * 60 * 1e3 // 5 minutes in ms

// inlined from @hoprnet/hopr-utils to remove dependency on whole package
const pickVersion = (full_version: string): string => {
  const split = full_version.split('.')
  return split[0] + '.' + split[1] + '.0'
}

const closures: {
  [key in DeploymentTypes]: number
} = {
  testing: shortDuration,
  development: shortDuration,
  staging: longDuration,
  production: longDuration
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

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
    salt: ethers.utils.formatBytes32String(salt),
    ...deployOptions
  })

  await result.deploy()
}

main.dependencies = ['preDeploy', 'HoprToken']
main.tags = ['HoprChannels']

export default main
