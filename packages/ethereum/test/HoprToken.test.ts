import { expectRevert } from '@openzeppelin/test-helpers'
import { HoprTokenContract, HoprTokenInstance } from '../types/truffle-contracts'

const HoprToken: HoprTokenContract = artifacts.require('HoprToken')

contract('HoprToken', function ([owner, userA]) {
  let hoprToken: HoprTokenInstance

  before(async function () {
    hoprToken = await HoprToken.deployed()
  })

  it("should be named 'HOPR Token'", async function () {
    expect(await hoprToken.name()).to.be.equal('HOPR Token', 'wrong name')
  })

  it("should have symbol 'HOPR'", async function () {
    expect(await hoprToken.symbol()).to.be.equal('xHOPR', 'wrong symbol')
  })

  it("should have a supply of '0'", async function () {
    const totalSupply = await hoprToken.totalSupply()

    expect(totalSupply.isZero()).to.be.equal(true, 'wrong total supply')
  })

  it('should fail mint', async function () {
    await expectRevert(
      hoprToken.mint(userA, 1, '0x00', '0x00', {
        from: userA,
      }),
      'HoprToken: caller does not have minter role'
    )
  })

  it("'owner' should be a minter", async function () {
    const minterRole = await hoprToken.MINTER_ROLE()

    expect(await hoprToken.hasRole(minterRole, owner)).to.be.equal(true, 'wrong minter')
  })

  it(`should mint 100 HOPR for 'owner'`, async function () {
    const amount = web3.utils.toWei('1', 'ether')

    await hoprToken.mint(owner, amount, '0x00', '0x00', {
      from: owner,
    })

    const balance = await hoprToken.balanceOf(owner).then((res) => res.toString())

    expect(balance).to.be.eq(amount, 'wrong balance')
  })
})
