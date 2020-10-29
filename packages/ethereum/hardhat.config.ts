// load env variables
require('dotenv').config()
// load hardhat plugins
import 'hardhat-typechain'
import '@nomiclabs/hardhat-truffle5'
import '@nomiclabs/hardhat-etherscan'

import { HardhatUserConfig, task } from 'hardhat/config'
import { mapValues } from 'lodash'
import { getRpcOptions } from './scripts/utils/networks'
import migrate from './scripts/migrate'

const { PRIVATE_KEY, INFURA, MATIC_VIGIL, ETHERSCAN } = process.env

const publicNetworks: HardhatUserConfig['networks'] = mapValues(
  getRpcOptions({ infura: INFURA, maticvigil: MATIC_VIGIL }),
  (config) =>
    ({
      chainId: config.chainId,
      url: config.httpUrl,
      gasMultiplier: 1.1,
      accounts: [PRIVATE_KEY]
    } as HardhatUserConfig['networks']['hardhat'])
)

const hardhatConfig: HardhatUserConfig = {
  defaultNetwork: 'hardhat',
  networks: {
    hardhat: {
      accounts: {
        // specify truffle's default mnemonic as we are expecting it in various areas in our codebase
        mnemonic: 'candy maple cake sugar pudding cream honey rich smooth crumble sweet treat'
      }
    },
    ...publicNetworks
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
    artifacts: './hardhat/artifacts'
  },
  typechain: {
    outDir: './scripts/utils/typechain',
    target: 'truffle-v5'
  },
  etherscan: {
    apiKey: ETHERSCAN
  }
}

task('migrate', 'Migrate contracts').setAction(migrate)

export default hardhatConfig
