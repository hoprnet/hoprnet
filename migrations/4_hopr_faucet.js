const HoprToken = artifacts.require('HoprToken')
const HoprFaucet = artifacts.require('HoprFaucet')

module.exports = async (deployer, network, [account]) => {
  const token = await HoprToken.deployed()

  if (['development', 'test', 'rinkeby', 'kovan'].includes(network)) {
    await deployer.deploy(HoprFaucet, token.address)
  }

  // renounce all roles and give minter role to HoprFaucet
  if (['rinkeby', 'kovan'].includes(network)) {
    const hoprFaucet = await HoprFaucet.deployed()
    const adminRole = await token.DEFAULT_ADMIN_ROLE()
    const minterRole = await token.MINTER_ROLE()

    await token.grantRole(minterRole, hoprFaucet.address)
    await token.revokeRole(minterRole, account)
    await token.revokeRole(adminRole, account)
  }
}
