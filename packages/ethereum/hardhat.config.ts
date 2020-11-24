// load env variables
require('dotenv').config()
// load hardhat plugins
import 'hardhat-typechain'
import 'hardhat-deploy'
import '@nomiclabs/hardhat-truffle5'
import '@nomiclabs/hardhat-etherscan'
import '@nomiclabs/hardhat-solhint'

import { HardhatUserConfig, task, types } from 'hardhat/config'
import { NODE_SEEDS, BOOTSTRAP_SEEDS } from '@hoprnet/hopr-demo-seeds'
import Web3 from 'web3'
import { MigrationOptions, getRpcOptions } from './utils/networks'

const { PRIVATE_KEY, INFURA, MATIC_VIGIL, ETHERSCAN } = process.env
const GAS_MULTIPLIER = 1.1

const devSeeds = NODE_SEEDS.concat(BOOTSTRAP_SEEDS).map((privateKey) => ({
  privateKey,
  balance: Web3.utils.toWei('10000', 'ether')
}))

const hardhatConfig: HardhatUserConfig = {
  defaultNetwork: 'localhost',
  networks: {
    hardhat: {
      live: false,
      tags: ['test', 'local'],
      accounts: devSeeds
    },
    localhost: {
      live: false,
      tags: ['local'],
      url: 'http://localhost:8545',
      accounts: devSeeds.map(({ privateKey }) => privateKey)
    },
    mainnet: {
      live: true,
      tags: [],
      chainId: 1,
      gasMultiplier: GAS_MULTIPLIER,
      url: `https://mainnet.infura.io/v3/${INFURA}`,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : []
    },
    kovan: {
      live: true,
      tags: ['staging'],
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
    }
  },
  namedAccounts: {
    deployer: 0,
    singleFaucetMinter: '0x1A387b5103f28bc6601d085A3dDC878dEE631A56'
  },
  solidity: {
    version: '0.6.6',
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
  etherscan: {
    apiKey: ETHERSCAN
  }
}

// create our own migration task since there isn't one implemented
// see https://github.com/nomiclabs/hardhat/issues/381
task('migrate', 'Migrate contracts', async (...args: any[]) => {
  // lazy load this as it breaks hardhat due to '@openzeppelin/test-helpers'
  // also required because we need to build typechain first
  return (await import('./tasks/migrate')).default(args[0], args[1], args[2])
})
  .addOptionalParam<MigrationOptions['shouldVerify']>(
    'shouldVerify',
    'Try to verify contracts using etherscan',
    false,
    types.boolean
  )
  .addOptionalParam<MigrationOptions['mintUsing']>(
    'mintUsing',
    'Mint using "minter" or "faucet"',
    'minter',
    types.string
  )
  .addOptionalParam<MigrationOptions['revokeRoles']>(
    'revokeRoles',
    'Revoke admin roles from deployer',
    false,
    types.boolean
  )

task('fund', 'Fund demo accounts', async (...args: any[]) => {
  return (await import('./tasks/fund')).default(args[0], args[1], args[2])
})
  .addParam<string>('address', 'HoprToken contract address', undefined, types.string)
  .addOptionalParam<string>('amount', 'Amount of HOPR to fund', Web3.utils.toWei('1000000', 'ether'), types.string)
  .addOptionalParam<number>('accountsToFund', 'Amount of accounts to fund from demo seeds', 0, types.int)

task('extract', 'Extract ABIs to specified folder', async (...args: any[]) => {
  return (await import('./tasks/extract')).default(args[0], args[1], args[2])
}).addFlag('target', 'Folder to output contents to')

export default hardhatConfig
