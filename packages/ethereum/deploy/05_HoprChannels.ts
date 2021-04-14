import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../chain'
import { durations } from '@hoprnet/hopr-utils'
import { storeContract } from './utils'

const closures: {
  [key in DeploymentTypes]: number
} = {
  local: durations.minutes(1),
  staging: durations.minutes(10),
  production: durations.minutes(60)
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const deploymentType = Object.keys(network.tags).find((tag) => closures[tag])

  const hoprToken = await deployments.get('HoprToken')

  const result = await deployments.deploy('HoprChannels', {
    from: deployer.address,
    args: [hoprToken.address, Math.floor((closures[deploymentType] ?? closures.local) / 1e3)],
    log: true
  })

  if (network.name !== 'hardhat') {
    await storeContract(network.name, 'HoprChannels', result.address, result.receipt.blockNumber)
  }
}

export default main
