import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'
import { getHoprStakeContractName } from '../utils/constants'

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, environment, maxFeePerGas, maxPriorityFeePerGas } = hre
  const { deploy } = deployments
  const { deployer, admin } = await getNamedAccounts()

  const HoprBoost = await deployments.get('HoprBoost')
  // xHOPR can be a contract automatically deployed by the bridge (on xDAI/Gnosis chain)
  const xHOPR = await deployments.get('xHoprToken')
  const wxHOPR = await deployments.get('HoprToken')

  // check the lastest block timestamp
  const latestBlockTimestamp = (await ethers.provider.getBlock('latest')).timestamp
  console.log(`Latest block timestamp is ${latestBlockTimestamp}`)
  const stakeContractName = getHoprStakeContractName(latestBlockTimestamp)
  const deployOptions = {
    log: true
  }
  // don't wait when using local hardhat because its using auto-mine
  if (!environment.match('hardhat')) {
    deployOptions['waitConfirmations'] = 2
  }

  const isStakingSeasonFiveAndAbove = parseInt([...stakeContractName.matchAll(/HoprStakeSeason(\d+)/g)][0][1], 10) > 4

  await deploy('HoprStake', {
    contract: stakeContractName,
    from: deployer,
    // beyond staking season 4 (from S5 on, no need to pass constructor parameters)
    args: isStakingSeasonFiveAndAbove ? [] : [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
    maxFeePerGas,
    maxPriorityFeePerGas,
    ...deployOptions
  })
}

main.tags = ['HoprStake']
main.dependencies = ['preDeploy', 'HoprBoost', 'HoprToken']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main
