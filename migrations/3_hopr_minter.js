const { durations } = require('@hoprnet/hopr-utils')
const networks = require('../truffle-networks')
const HoprMinter = artifacts.require('HoprMinter')
const HoprToken = artifacts.require('HoprToken')

module.exports = async (deployer, network, [account]) => {
  const config = networks[network.replace('-fork', '')]
  const token = await HoprToken.deployed()
  const maxAmount = web3.utils.toWei('100000000', 'ether')
  const duration = Math.floor(durations.days(365) / 1e3)

  // deploy HoprMinter only on dev networks & mainnet network
  if (config.network_type === 'development' || config.network_type === 'mainnet') {
    await deployer.deploy(HoprMinter, token.address, maxAmount, duration)
  }

  // renounce all roles and give minter role to HoprMinter
  if (config.network_type === 'mainnet') {
    const hoprMinter = await HoprMinter.deployed()
    const adminRole = await token.DEFAULT_ADMIN_ROLE()
    const minterRole = await token.MINTER_ROLE()

    await token.grantRole(minterRole, hoprMinter.address)
    await token.renounceRole(minterRole, account)
    await token.renounceRole(adminRole, account)
  }
}
