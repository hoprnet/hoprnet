const { durations } = require('@hoprnet/hopr-utils')

const HoprChannels = artifacts.require('HoprChannels')
const HoprToken_3 = artifacts.require('HoprToken')

module.exports = async (deployer) => {
  const token = await HoprToken_3.deployed()
  const secsClosure = Math.floor(durations.days(2) / 1e3)

  await deployer.deploy(HoprChannels, token.address, secsClosure)
}
