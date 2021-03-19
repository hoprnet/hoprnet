import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import type { DeploymentTypes } from '../chain'
import Web3 from 'web3'
import { durations } from '@hoprnet/hopr-utils'

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
  local: Web3.utils.toWei('100000000', 'ether'),
  staging: Web3.utils.toWei('100000000', 'ether'),
  production: Web3.utils.toWei('100000000', 'ether')
}

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network } = hre
  const { deploy } = deployments
  const { deployer } = await getNamedAccounts()
  const deploymentType = Object.keys(network.tags).find((tag) => startTimes[tag])

  const hoprToken = await deployments.get('HoprToken')

  await deploy('HoprDistributor', {
    from: deployer,
    args: [
      hoprToken.address,
      Math.floor(startTimes[deploymentType] ?? startTimes.local / 1e3),
      maxMintAmounts[deploymentType] ?? maxMintAmounts.local
    ],
    log: true
  })
}

export default main
