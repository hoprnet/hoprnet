import type { SolcUserConfig } from 'hardhat/types'
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
import { HardhatUserConfig, task, types } from 'hardhat/config'
import { ethers } from 'ethers'
import { networks, NetworkTag } from './constants'

const { DEPLOYER_WALLET_PRIVATE_KEY, ETHERSCAN_KEY, INFURA_KEY, QUIKNODE_KEY, DEVELOPMENT = false } = process.env
const GAS_MULTIPLIER = 1.1

// set 'ETHERSCAN_API_KEY' so 'hardhat-deploy' can read it
process.env.ETHERSCAN_API_KEY = ETHERSCAN_KEY

const hardhatConfig: HardhatUserConfig = {
  defaultNetwork: 'hardhat',
  networks: {
    // hardhat-deploy cannot run deployments if the network is not hardhat
    // we use an ENV variable (which is specified in our NPM script)
    // to let hardhat know we want to run hardhat in 'development' mode
    // this essentially enables mining, see below
    hardhat: {
      live: false,
      tags: [DEVELOPMENT ? 'development' : 'testing'] as NetworkTag[],
      saveDeployments: true,
      mining: DEVELOPMENT
        ? {
            auto: true, // every transaction will trigger a new block (without this deployments fail)
            interval: [1000, 3000] // mine new block every 1 - 3s
          }
        : undefined
    },
    goerli: {
      ...networks.goerli,
      live: true,
      tags: ['staging'] as NetworkTag[],
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://goerli.infura.io/v3/${INFURA_KEY}`,
      accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
    },
    xdai: {
      ...networks.xdai,
      live: true,
      tags: ['production'] as NetworkTag[],
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://still-patient-forest.xdai.quiknode.pro/${QUIKNODE_KEY}/`,
      accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
    },
    mumbai: {
      ...networks.mumbai,
      live: false,
      tags: ['testing'] as NetworkTag[],
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://polygon-mumbai.infura.io/v3/${INFURA_KEY}`,
      accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
    },
    polygon: {
      ...networks.polygon,
      live: false,
      tags: ['testing'] as NetworkTag[],
      url: `https://polygon-mainnet.infura.io/v3/${INFURA_KEY}`,
      accounts: DEPLOYER_WALLET_PRIVATE_KEY ? [DEPLOYER_WALLET_PRIVATE_KEY] : []
    }

  },
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
    deployments: './deployments'
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
