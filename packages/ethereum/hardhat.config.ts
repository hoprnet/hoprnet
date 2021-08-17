import type { HardhatRuntimeEnvironment, HardhatConfig, SolcUserConfig } from 'hardhat/types'
// load env variables
require('dotenv').config()
// load hardhat plugins
import '@nomiclabs/hardhat-ethers'
import '@nomiclabs/hardhat-solhint'
import '@nomiclabs/hardhat-waffle'
import 'hardhat-deploy'
import 'hardhat-gas-reporter'
import 'solidity-coverage'
import '@typechain/hardhat'
// rest
import { HardhatUserConfig, task, types, extendEnvironment, extendConfig } from 'hardhat/config'
import { ethers } from 'ethers'
import type { Network } from '@hoprnet/hopr-core'
export type DeploymentTypes = 'testing' | 'development' | 'staging' | 'production'
export type NetworkTag = DeploymentTypes | 'etherscan'

const {
  DEPLOYER_WALLET_PRIVATE_KEY,
  ETHERSCAN_KEY,
  ENVIRONMENT_ID = 'default'
} = process.env

extendConfig((config: HardhatConfig) => {
  config.etherscan.apiKey = ETHERSCAN_KEY
})

extendEnvironment((hre: HardhatRuntimeEnvironment) => {
  hre.environment = ENVIRONMENT_ID
})

const PROTOCOL_CONFIG = require('../hoprd/protocol-config.json')

// // TODO - I don't understand this
// const NETWORKS = {
//   // hardhat-deploy cannot run deployments if the network is not hardhat
//   // we use an ENV variable (which is specified in our NPM script)
//   // to let hardhat know we want to run hardhat in 'development' mode
//   // this essentially enables mining, see below
//   hardhat: {
//     live: false,
//     tags: [DEVELOPMENT ? 'development' : 'testing'] as NetworkTag[],
//     saveDeployments: true,
//     mining: DEVELOPMENT
//       ? {
//         auto: true, // every transaction will trigger a new block (without this deployments fail)
//         interval: [1000, 3000] // mine new block every 1 - 3s
//       }
//       : undefined
//   }
// }

// Object.keys(ENVIRONMENTS).forEach(k => {
//   const env = ENVIRONMENTS[k]
//   NETWORKS[k] = {
//     live: env.live,
//     tags: env.tags as NetworkTag[],
//     gasMultiplier: env['gas-multiplier'],
//     url: env['default-provider'], // TODO key substitution,
//     accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
//   }
// })

function networkToHardhatNetwork(input: Network): any {
  return {
    live: true, // @TODO
    tags: [],
    gasMultiplier: input.gas_multiplier,
    url: input.default_provider,
    accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
  }
}


const networks = {}

for (const network of PROTOCOL_CONFIG.networks) {
  const hardhatNetwork = networkToHardhatNetwork(network)
  networks[network.id] = hardhatNetwork
}

console.log(networks)

const hardhatConfig: HardhatUserConfig = {
  defaultNetwork: 'hardhat',
  networks,
  namedAccounts: {
    deployer: 0
  },
  solidity: {
    compilers: ['0.8.3', '0.6.6', '0.4.24'].map<SolcUserConfig>((version) => ({
      version,
      settings: {
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    }))
  },
  paths: {
    sources: './contracts',
    tests: './test',
    cache: './hardhat/cache',
    artifacts: './hardhat/artifacts',
    deployments: `./deployments/${ENVIRONMENT_ID}`
  },
  typechain: {
    outDir: './types',
    target: 'ethers-v5'
  },
  gasReporter: {
    currency: 'USD',
    excludeContracts: ['mocks', 'utils/console.sol']
  }
}

task('faucet', 'Faucets a local development HOPR node account with ETH and HOPR tokens', async (...args: any[]) => {
  return (await import('./tasks/faucet')).default(args[0], args[1], args[2])
})
  .addParam<string>('address', 'HoprToken address', undefined, types.string)
  .addOptionalParam<string>('amount', 'Amount of HOPR to fund', ethers.utils.parseEther('1').toString(), types.string)
  .addOptionalParam<boolean>(
    'ishopraddress',
    'Whether the address passed is a HOPR address or not',
    false,
    types.boolean
  )

task('accounts', 'View unlocked accounts', async (...args: any[]) => {
  return (await import('./tasks/getAccounts')).default(args[0], args[1], args[2])
})

export default hardhatConfig
