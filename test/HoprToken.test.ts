import { HoprTokenContract, HoprTokenInstance } from '../types/truffle-contracts'

const HoprToken: HoprTokenContract = artifacts.require('HoprToken')

contract('HoprToken', function([owner]) {
  let hoprToken: HoprTokenInstance

  before(async function() {
    hoprToken = await HoprToken.deployed()
  })

  it("should be named 'HOPR Token'", async function() {
    expect(await hoprToken.name()).to.be.equal('HOPR Token', 'wrong name')
  })

  it("should have symbol 'HOPR'", async function() {
    expect(await hoprToken.symbol()).to.be.equal('HOPR', 'wrong symbol')
  })

  it("should have a supply of '0'", async function() {
    const totalSupply = await hoprToken.totalSupply()

    expect(totalSupply.isZero()).to.be.equal(true, 'wrong total supply')
  })

  it("'owner' should be a minter", async function() {
    expect(await hoprToken.isMinter(owner)).to.be.equal(true, 'wrong minter')
  })
})
