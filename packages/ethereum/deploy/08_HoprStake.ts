import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'

const S3_PROGRAM_END = 1658836800
const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

const main: DeployFunction = async function ({
  ethers,
  deployments,
  network,
  getNamedAccounts,
  environment
}: HardhatRuntimeEnvironment) {
  const { deploy } = deployments
  const { deployer, admin } = await getNamedAccounts()
  const environmentConfig = PROTOCOL_CONFIG.environments[environment]

  const HoprBoost = await deployments.get('HoprBoost')
  // xHOPR can be a contract automatically deployed by the bridge (on xDAI/Gnosis chain)
  const xHOPR =
    network.tags.testing || network.tags.development
      ? await deployments.get('xHoprMock')
      : environmentConfig['minted_token_receiver_address']
  const wxHOPR = await deployments.get('HoprToken')

  // check the lastest block timestamp
  const latestBlockTimestamp = (await ethers.provider.getBlock('latest')).timestamp
  console.log(`Latest block timestamp is ${latestBlockTimestamp}`)

  if (latestBlockTimestamp <= S3_PROGRAM_END) {
    // deploy season 3
    await deploy('HoprStake', {
      contract: 'HoprStakeSeason3',
      from: deployer,
      args: [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
      log: true
    })
  } else {
    // deploy season 4
    await deploy('HoprStake', {
      contract: 'HoprStakeSeason4',
      from: deployer,
      args: [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
      log: true
    })
  }
}

main.tags = ['HoprStake']
main.dependencies = ['preDeploy', 'HoprBoost', 'HoprToken']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main
