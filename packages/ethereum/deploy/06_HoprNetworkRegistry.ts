import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import { DeploymentTypes } from '../src'
import { INITIAL_MIN_STAKE } from '../test/HoprNetworkRegistry.test'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')
const minStakes: {
  [key in DeploymentTypes]: number
} = {
  testing: INITIAL_MIN_STAKE,
  development: 0,
  staging: 0,
  production: 0
}
// Deploy directly a whitelist contract, using hardcoded staking contract.
const main = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre

  const environmentConfig = PROTOCOL_CONFIG.environments[environment]
  const deployer = await getNamedAccounts().then((o) => ethers.getSigner(o.deployer))
  const deploymentType = Object.keys(network.tags).find((tag) => minStakes[tag])

  const adminAddress =
    network.name == 'hardhat' ? deployer.address : environmentConfig['network_registry_admin_address']

  console.log(JSON.stringify(environmentConfig))

  await deployments.deploy('HoprNetworkRegistry', {
    from: deployer.address,
    args: [environmentConfig['stake_v2_contract_address'], adminAddress, minStakes[deploymentType]]
  })
}

main.dependencies = ['preDeploy']
main.tags = ['HoprNetworkRegistry']

export default main
