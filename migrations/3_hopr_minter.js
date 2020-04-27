const { durations } = require('@hoprnet/hopr-utils')

const HoprMinter = artifacts.require('HoprMinter')
const HoprToken_4 = artifacts.require('HoprToken')

module.exports = async (deployer) => {
  const token = await HoprToken_4.deployed()
  const maxAmount = web3.utils.toWei('100000000', 'ether')
  const duration = durations.days(365) / 1e3

  await deployer.deploy(HoprMinter, token.address, maxAmount, duration)
}
