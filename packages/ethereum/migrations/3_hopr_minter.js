const { durations } = require('@hoprnet/hopr-utils')
const networks = require('../truffle-networks')
const HoprMinter = artifacts.require('HoprMinter')
const HoprToken = artifacts.require('HoprToken')

module.exports = async (deployer, _network, [owner]) => {
  const network = _network.replace('-fork', '')
  const config = networks[network]
  const hoprToken = await HoprToken.deployed()
  const maxAmount = web3.utils.toWei('100000000', 'ether')
  const duration = Math.floor(durations.days(365) / 1e3)

  let hoprMinter
  let adminRole
  let minterRole

  // deploy HoprMinter only on development networks & mainnet network
  if (config.network_type === 'development' || config.network_type === 'mainnet') {
    await deployer.deploy(HoprMinter, hoprToken.address, maxAmount, duration)

    hoprMinter = await HoprMinter.deployed()
    adminRole = await hoprToken.DEFAULT_ADMIN_ROLE()
    minterRole = await hoprToken.MINTER_ROLE()
  }

  // renounce all roles and give minter role to contract HoprMinter
  if (config.network_type === 'mainnet') {
    await hoprToken.grantRole(minterRole, hoprMinter.address)
    await hoprToken.renounceRole(minterRole, owner)
    await hoprToken.renounceRole(adminRole, owner)
  }
}
