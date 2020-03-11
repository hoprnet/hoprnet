require("ts-node/register");
const HDWalletProvider = require("@truffle/hdwallet-provider");

let secrets;
try {
  secrets = require("./truffle-secrets.json");
} catch (error) {
  console.log("truffle-secrets not found!");
}

module.exports = {
  networks: {
    // default network
    // migrations mint 100 HOPR to 'owner'
    development: {
      host: "127.0.0.1",
      port: 9545, // 'truffle develop' port
      network_id: "*"
    },

    // used when testing
    // migrations replicate production
    test: {
      host: "127.0.0.1",
      port: 9545, // 'truffle develop' port
      network_id: "*"
    },

    coverage: {
      host: "localhost",
      network_id: "*",
      port: 8555, // if you change this, also set the port option in '.solcover.js'
      gas: 0xfffffffffff, // high gas value
      gasPrice: 0x01 // low gas price
    },

    rinkeby: secrets && {
      provider: () =>
        new HDWalletProvider(
          secrets.mnemonic,
          `https://rinkeby.infura.io/v3/${secrets.infura}`
        ),
      gas: 6500000,
      gasPrice: 1000000000,
      network_id: "4"
    }
  },

  // default mocha options
  mocha: {
    timeout: 100000
  },

  // configure your compilers
  compilers: {
    solc: {
      version: "0.5.7", // Fetch exact version from solc-bin (default: truffle's version)
      docker: false, // Use "0.5.7" you've installed locally with docker (default: false)
      settings: {
        // See the solidity docs for advice about optimization and evmVersion
        optimizer: {
          enabled: true,
          runs: 200
        }
      }
    }
  }
};
