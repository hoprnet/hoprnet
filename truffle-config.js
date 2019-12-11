require("ts-node/register");

module.exports = {
  networks: {
    // default network
    development: {
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
    }
  },

  // default mocha options
  mocha: {
    timeout: 100000
  },

  // configure your compilers
  compilers: {
    solc: {
      version: "0.5.3", // Fetch exact version from solc-bin (default: truffle's version)
      docker: false, // Use "0.5.3" you've installed locally with docker (default: false)
      settings: {
        // See the solidity docs for advice about optimization and evmVersion
        optimizer: {
          enabled: true,
          runs: 200
        },
        evmVersion: "byzantium"
      }
    }
  }
};
