import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../chain'
import { durations } from '@hoprnet/hopr-utils'
import Ethers from 'ethers'

const startTimes: {
  [key in DeploymentTypes]: number
} = {
  local: durations.days(1),
  staging: durations.days(1),
  production: durations.days(1)
}

const maxMintAmounts: {
  [key in DeploymentTypes]: string
} = {
  local: Ethers.utils.parseEther('100000000').toString(),
  staging: Ethers.utils.parseEther('100000000').toString(),
  production: Ethers.utils.parseEther('100000000').toString()
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
      Math.floor(startTimes[deploymentType] ?? startTimes.local / 1e3),
      maxMintAmounts[deploymentType] ?? maxMintAmounts.local
    ],
    log: true
  })
}

export default main
