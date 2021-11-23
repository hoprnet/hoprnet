import type { HardhatRuntimeEnvironment } from 'hardhat/types'
import type { DeployFunction } from 'hardhat-deploy/types'

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

// runs before any deployment
const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const environment = PROTOCOL_CONFIG.environments[hre.environment]
  if (!(hre.environment in PROTOCOL_CONFIG.environments)) {
    console.log(`Error: specified environment ${hre.environment} not supported`)
    process.exit(1)
  }

  if (environment.network_id !== hre.network.name) {
    console.log(
      `Error: specified environment ${hre.environment} and network ${hre.network.name} not supported together`
    )
    process.exit(1)
  }
}

main.tags = ['preDeploy']

export default main
