import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../src/constants'
import { durations } from '@hoprnet/hopr-utils'
import { ethers } from 'ethers'

const startTimes: {
  [key in DeploymentTypes]: number
} = {
  testing: durations.days(1),
  development: durations.days(1),
  staging: durations.days(1),
  production: durations.days(1)
}

const maxMintAmounts: {
  [key in DeploymentTypes]: string
} = {
  testing: ethers.utils.parseEther('100000000').toString(),
  development: ethers.utils.parseEther('100000000').toString(),
  staging: ethers.utils.parseEther('100000000').toString(),
  production: ethers.utils.parseEther('100000000').toString()
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network } = hre
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const deploymentType = Object.keys(network.tags).find((tag) => startTimes[tag])

  const hoprToken = await deployments.get('HoprToken')

  await deployments.deploy('HoprDistributor', {
    from: deployer.address,
    args: [
      hoprToken.address,
      Math.floor(startTimes[deploymentType] ?? startTimes.testing / 1e3),
      maxMintAmounts[deploymentType] ?? maxMintAmounts.testing
    ],
    log: true
  })
}

// this smart contract should not be redeployed on a production network
main.skip = async (env) => !!env.network.tags.production

export default main
