require('ts-node/register')
const HDWalletProvider = require('@truffle/hdwallet-provider')
const networks = require('./truffle-networks.json')

let secrets
try {
  secrets = require('./truffle-secrets.json')
} catch (error) {
  console.warn('truffle-secrets not found!')
}

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

    rinkeby: secrets && {
      ...networks.rinkeby,
      provider: () => new HDWalletProvider(secrets.mnemonic, `https://rinkeby.infura.io/v3/${secrets.infura}`),
    },

    kovan: secrets && {
      ...networks.kovan,
      provider: () => new HDWalletProvider(secrets.mnemonic, `https://kovan.infura.io/v3/${secrets.infura}`),
    },
  },

  // default mocha options
  mocha: {
    timeout: 100000,
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

  api_keys: secrets
    ? {
        etherscan: secrets.etherscan,
      }
    : undefined,

  plugins: ['solidity-coverage', 'truffle-plugin-verify'],
}
