import { deployments } from 'hardhat'
import { singletons, expectRevert } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from './utils'

const HoprToken = artifacts.require('HoprToken')

const useFixtures = deployments.createFixture(async () => {
  const [deployer, userA] = await web3.eth.getAccounts()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer)

  // deploy ChannelsMock
  const token = await HoprToken.new()

  // allow deployet to mint tokens
  await token.grantRole(await token.MINTER_ROLE(), deployer)

  return {
    token,
    deployer,
    userA
  }
})

describe('HoprToken', function () {
  it("should be named 'HOPR Token'", async function () {
    const { token } = await useFixtures()

    expect(await token.name()).to.be.equal('HOPR Token', 'wrong name')
  })

  it("should have symbol 'HOPR'", async function () {
    const { token } = await useFixtures()

    expect(await token.symbol()).to.be.equal('HOPR', 'wrong symbol')
  })

  it("should have a supply of '0'", async function () {
    const { token } = await useFixtures()

    const totalSupply = await token.totalSupply()
    expect(totalSupply.isZero()).to.be.equal(true, 'wrong total supply')
  })

  it('should fail mint', async function () {
    const { token, userA } = await useFixtures()
    await expectRevert(
      token.mint(userA, 1, '0x00', '0x00', {
        from: userA
      }),
      vmErrorMessage('caller does not have minter role')
    )
  })

  it("'deployer' should be a minter", async function () {
    const { token, deployer } = await useFixtures()
    const minterRole = await token.MINTER_ROLE()

    expect(await token.hasRole(minterRole, deployer)).to.be.equal(true, 'wrong minter')
  })

  it(`should mint 100 HOPR for 'deployer'`, async function () {
    const { token, deployer } = await useFixtures()
    const amount = web3.utils.toWei('1', 'ether')

    await token.mint(deployer, amount, '0x00', '0x00', {
      from: deployer
    })

    const balance = await token.balanceOf(deployer).then((res) => res.toString())

    expect(balance).to.be.eq(amount, 'wrong balance')
  })
})
