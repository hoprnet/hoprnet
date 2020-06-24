require('@openzeppelin/test-helpers/configure')({ provider: web3.currentProvider, environment: 'truffle' })
const { singletons } = require('@openzeppelin/test-helpers')
const networks = require('../truffle-networks')
const HoprToken = artifacts.require('HoprToken')

module.exports = async (deployer, network, [owner]) => {
  const config = networks[network.replace('-fork', '')]

  if (config.network_type === 'development') {
    // in a local environment an ERC777 token requires deploying an ERC1820 registry
    await singletons.ERC1820Registry(owner)
  }

  await deployer.deploy(HoprToken)
}
