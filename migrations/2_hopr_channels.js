require('@openzeppelin/test-helpers/configure')({ provider: web3.currentProvider, environment: 'truffle' })
const { durations } = require('@hoprnet/hopr-utils')
const HoprChannels = artifacts.require('HoprChannels')
const HoprToken = artifacts.require('HoprToken')

module.exports = async (deployer) => {
  const token = await HoprToken.deployed()
  const secsClosure = Math.floor(durations.minutes(1) / 1e3)

  await deployer.deploy(HoprChannels, token.address, secsClosure)
}
