const HoprToken = artifacts.require('HoprToken')

require('@openzeppelin/test-helpers/configure')({ provider: web3.currentProvider, environment: 'truffle' })
const { singletons } = require('@openzeppelin/test-helpers')

module.exports = async (deployer, network, [owner]) => {
  if (['development', 'test', 'soliditycoverage'].includes(network)) {
    // in a local environment an ERC777 token requires deploying an ERC1820 registry
    await singletons.ERC1820Registry(owner)
  }

  await deployer.deploy(HoprToken)
}
