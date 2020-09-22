const networks = require('../truffle-networks')
const HoprToken = artifacts.require('HoprToken')
const HoprFaucet = artifacts.require('HoprFaucet')

const XDAI_FAUCET_ADDRESS = '0x1A387b5103f28bc6601d085A3dDC878dEE631A56'

module.exports = async (deployer, _network, [owner]) => {
  const network = _network.replace('-fork', '')
  const config = networks[network]
  const singleFaucetUser = ['xdai'].includes(network)
  const hoprToken = await HoprToken.deployed()

  let hoprFaucet
  let pauserRole
  let minterRole

  // deploy HoprFaucet only on development networks & testnet networks
  if (config.network_type === 'development' || config.network_type === 'testnet') {
    await deployer.deploy(HoprFaucet, hoprToken.address, singleFaucetUser)

    hoprFaucet = await HoprFaucet.deployed()
    pauserRole = await hoprFaucet.PAUSER_ROLE()
    minterRole = await hoprFaucet.MINTER_ROLE()
  }

  // give contract HoprFaucet MINTER_ROLE
  if (config.network_type === 'development') {
    await hoprFaucet.grantRole(pauserRole, owner)
    await hoprFaucet.grantRole(minterRole, hoprFaucet.address)
  }
  // give 'owner' OR 'XDAI_FAUCET_ADDRESS' MINTER_ROLE and PAUSER_ROLE
  else if (config.network_type === 'testnet') {
    if (singleFaucetUser) {
      await hoprFaucet.grantRole(pauserRole, owner)
      await hoprFaucet.grantRole(minterRole, XDAI_FAUCET_ADDRESS)
    } else {
      await hoprFaucet.grantRole(pauserRole, owner)
      await hoprFaucet.grantRole(minterRole, hoprFaucet.address)
    }

    await hoprToken.grantRole(minterRole, hoprFaucet.address)
  }
}
