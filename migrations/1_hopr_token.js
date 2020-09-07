require('@openzeppelin/test-helpers/configure')({ provider: web3.currentProvider, environment: 'truffle' })
const { singletons } = require('@openzeppelin/test-helpers')
const networks = require('../truffle-networks')
const HoprToken = artifacts.require('HoprToken')

module.exports = async (deployer, _network, [owner]) => {
  const network = _network.replace('-fork', '')
  const config = networks[network]

  if (config.network_type === 'development') {
    // in a local environment an ERC777 token requires an ERC1820 registry
    await singletons.ERC1820Registry(owner)
  }

  await deployer.deploy(HoprToken)

  const hoprToken = await HoprToken.deployed()
  const minterRole = await hoprToken.MINTER_ROLE()

  // give owner 'MINTER_ROLE' if we are running this on 'development'
  if (config.network_type === 'development') {
    await hoprToken.grantRole(minterRole, owner)
  }
}
