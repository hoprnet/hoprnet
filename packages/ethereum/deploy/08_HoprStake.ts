import { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeployFunction } from 'hardhat-deploy/types'
import { getHoprStakeContractName } from '../utils/constants'

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

  const stakeContractName = getHoprStakeContractName(latestBlockTimestamp)
  await deploy('HoprStake', {
    contract: stakeContractName,
    from: deployer,
    args: [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
    log: true
  })
}

main.tags = ['HoprStake']
main.dependencies = ['HoprBoost', 'HoprToken']
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main
