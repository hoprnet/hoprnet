require('ts-node/register')
require('dotenv').config()
const process = require('process')
const HDWalletProvider = require('@truffle/hdwallet-provider')
const networks = require('./truffle-networks')

const { PRIVATE_KEY, ETHERSCAN, INFURA, MATIC } = process.env
const canMigrate = typeof PRIVATE_KEY !== 'undefined' && typeof INFURA !== 'undefined' && MATIC !== 'undefined'

module.exports = {
  networks: {
    // default network
    development: {
      ...networks.development,
    },

    // used when testing
    test: {
      ...networks.test,
    },

    // used for generating code coverage
    soliditycoverage: {
      ...networks.soliditycoverage,
    },

    rinkeby: canMigrate && {
      ...networks.rinkeby,
      provider: () => new HDWalletProvider(PRIVATE_KEY, `https://rinkeby.infura.io/v3/${INFURA}`),
    },

    kovan: canMigrate && {
      ...networks.kovan,
      provider: () => new HDWalletProvider(PRIVATE_KEY, `https://kovan.infura.io/v3/${INFURA}`),
    },

    // block explorer: https://blockscout.com/poa/sokol
    solkol: canMigrate && {
      ...networks.solkol,
      provider: () => new HDWalletProvider(PRIVATE_KEY, 'https://sokol.poa.network'),
    },

    // block explorer: https://blockscout.com/poa/core
    xdai: canMigrate && {
      ...networks.xdai,
      provider: () => new HDWalletProvider(PRIVATE_KEY, 'https://xdai.poanetwork.dev'),
    },

    // block explorer: https://explorer.matic.network
    matic: canMigrate && {
      ...networks.matic,
      provider: () => new HDWalletProvider(PRIVATE_KEY, `https://rpc-mainnet.maticvigil.com/v1/${MATIC}`),
    },
  },

  // default mocha options
  mocha: {
    timeout: 200e3,
  },

  // configure your compilers
  compilers: {
    solc: {
      version: '0.6.6', // Fetch exact version from solc-bin (default: truffle's version)
      settings: {
        // See the solidity docs for advice about optimization and evmVersion
        optimizer: {
          enabled: true,
          runs: 200,
        },
      },
    },
  },

  api_keys: {
    etherscan: ETHERSCAN,
  },

  plugins: ['solidity-coverage', 'truffle-plugin-verify'],
}
