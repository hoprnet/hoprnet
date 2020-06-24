const networks = require('../truffle-networks')
const HoprToken = artifacts.require('HoprToken')
const HoprFaucet = artifacts.require('HoprFaucet')

module.exports = async (deployer, network, [account]) => {
  const config = networks[network.replace('-fork', '')]
  const token = await HoprToken.deployed()

  // deploy HoprFaucet only on dev networks & testnet networks
  if (config.network_type === 'development' || config.network_type === 'testnet') {
    await deployer.deploy(HoprFaucet, token.address)
  }

  // renounce all roles and give minter role to HoprFaucet
  if (config.network_type === 'testnet') {
    const hoprFaucet = await HoprFaucet.deployed()
    const adminRole = await token.DEFAULT_ADMIN_ROLE()
    const minterRole = await token.MINTER_ROLE()

    await token.grantRole(minterRole, hoprFaucet.address)
    await token.revokeRole(minterRole, account)
    await token.revokeRole(adminRole, account)
  }
}
