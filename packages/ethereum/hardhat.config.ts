// load env variables
require('dotenv').config()
// load hardhat plugins
import 'hardhat-typechain'
import '@nomiclabs/hardhat-truffle5'
import '@nomiclabs/hardhat-etherscan'
import '@nomiclabs/hardhat-solhint'

import type { HardhatUserConfig } from 'hardhat/config'
import Web3 from 'web3'
import { mapValues } from 'lodash'
import { getRpcOptions } from './scripts/utils/networks'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'

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
      accounts: NODE_SEEDS.map((privateKey) => ({
        privateKey,
        balance: Web3.utils.toWei('10000', 'ether')
      }))
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
    outDir: './types',
    target: 'truffle-v5'
  },
  etherscan: {
    apiKey: ETHERSCAN
  }
}

export default hardhatConfig
