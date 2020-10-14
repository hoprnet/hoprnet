/**
 * @typedef {Object} Network
 * @property {string} network_id
 * @property {'development' | 'testnet' | 'mainnet'} network_type
 * @property {string=} host
 * @property {number=} port
 * @property {number=} gas
 * @property {number=} gasPrice
 * @property {boolean=} noVerify
 */

/** @type {Object.<string, Network>} */
module.exports = {
  development: {
    network_id: '*',
    network_type: 'development',
    host: '127.0.0.1',
    port: 9545,
  },
  test: {
    network_id: '*',
    network_type: 'development',
    host: '127.0.0.1',
    port: 9545,
  },
  soliditycoverage: {
    network_id: '*',
    network_type: 'development',
    host: '127.0.0.1',
    port: 8555,
  },
  rinkeby: {
    network_id: '4',
    network_type: 'testnet',
    gas: 6500000,
    gasPrice: 1000000000,
  },
  kovan: {
    network_id: '42',
    network_type: 'testnet',
    gas: 6500000,
    gasPrice: 1000000000,
  },
  solkol: {
    network_id: '77',
    network_type: 'testnet',
    gas: 6500000,
    gasPrice: 1000000000,
    noVerify: true,
  },
  xdai: {
    network_id: '100',
    network_type: 'testnet',
    gas: 6500000,
    gasPrice: 1000000000,
    noVerify: true,
  },
  matic: {
    network_id: '137',
    network_type: 'testnet',
    gas: 20000000,
    gasPrice: 1000000000,
    noVerify: true,
  },
}
