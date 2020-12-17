// load env variables
require('dotenv').config()
// load hardhat plugins
import 'hardhat-typechain'
import 'hardhat-deploy'
import '@nomiclabs/hardhat-truffle5'
import '@nomiclabs/hardhat-solhint'
import 'solidity-coverage'
import 'hardhat-gas-reporter'

import { HardhatUserConfig, task, types } from 'hardhat/config'
import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'
import Web3 from 'web3'
import { ACCOUNT_DEPLOYER_PRIVKEY, ACCOUNT_A_PRIVKEY, ACCOUNT_B_PRIVKEY } from './test/constants'

const { PRIVATE_KEY, INFURA, MATIC_VIGIL, ETHERSCAN } = process.env
const GAS_MULTIPLIER = 1.1

// set 'ETHERSCAN_API_KEY' so 'hardhat-deploy' can read it
process.env.ETHERSCAN_API_KEY = ETHERSCAN

// legacy: use hopr-demo-seeds
const localhostPrivKeys = NODE_SEEDS.concat(BOOTSTRAP_SEEDS)

// private keys used by tests
// @TODO: fix legacy dependancy
const hardhatPrivKeys = localhostPrivKeys
  .concat([ACCOUNT_DEPLOYER_PRIVKEY, ACCOUNT_A_PRIVKEY, ACCOUNT_B_PRIVKEY])
  .map((privateKey) => ({
    privateKey,
    balance: Web3.utils.toWei('10000', 'ether')
  }))

const hardhatConfig: HardhatUserConfig = {
  defaultNetwork: 'hardhat',
  networks: {
    hardhat: {
      live: false,
      tags: ['local', 'test'],
      accounts: hardhatPrivKeys,
      allowUnlimitedContractSize: true // TODO: investigate
    },
    localhost: {
      live: false,
      tags: ['local'],
      url: 'http://localhost:8545',
      accounts: localhostPrivKeys
    },
    mainnet: {
      live: true,
      tags: ['production', 'etherscan'],
      chainId: 1,
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://mainnet.infura.io/v3/${INFURA}`,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : []
    },
    kovan: {
      live: true,
      tags: ['staging', 'etherscan'],
      chainId: 42,
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://kovan.infura.io/v3/${INFURA}`,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : []
    },
    xdai: {
      live: true,
      tags: ['staging'],
      chainId: 100,
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://xdai.poanetwork.dev`,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : []
    },
    matic: {
      live: true,
      tags: ['staging'],
      chainId: 137,
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://rpc-mainnet.maticvigil.com/v1/${MATIC_VIGIL}`,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : []
    },
    binance: {
      live: true,
      tags: ['staging'],
      chainId: 56,
      url: 'https://bsc-dataseed.binance.org',
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : [],
      gas: Number(Web3.utils.toWei('20', 'gwei')) // binance chain requires >= 20gwei
    }
  },
  namedAccounts: {
    deployer: 0
  },
  solidity: {
    version: '0.7.5',
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
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
    target: 'truffle-v5'
  },
  gasReporter: {
    currency: 'USD',
    excludeContracts: ['mocks', 'Migrations.sol', 'utils/console.sol']
  }
}

task('fund', "Fund node's accounts by specifying HoprToken address", async (...args: any[]) => {
  return (await import('./tasks/fund')).default(args[0], args[1], args[2])
})
  .addParam<string>('address', 'HoprToken contract address', undefined, types.string)
  .addOptionalParam<string>('amount', 'Amount of HOPR to fund', Web3.utils.toWei('1000000', 'ether'), types.string)
  .addOptionalParam<number>('accountsToFund', 'Amount of accounts to fund from demo seeds', 0, types.int)

task('postCompile', 'Use export task and then update abis folder', async (...args: any[]) => {
  return (await import('./tasks/postCompile')).default(args[0], args[1])
})

task('postDeploy', 'Use export task and then update addresses folder', async (...args: any[]) => {
  return (await import('./tasks/postDeploy')).default(args[0], args[1])
})

export default hardhatConfig
