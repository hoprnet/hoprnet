import { expectRevert, expectEvent, constants } from '@openzeppelin/test-helpers'
import {
  HoprFaucetContract,
  HoprFaucetInstance,
  HoprTokenContract,
  HoprTokenInstance,
} from '../types/truffle-contracts'

const HoprToken: HoprTokenContract = artifacts.require('HoprToken')
const HoprFaucet: HoprFaucetContract = artifacts.require('HoprFaucet')

contract('HoprFaucet', function ([owner, userA]) {
  let hoprToken: HoprTokenInstance
  let hoprFaucet: HoprFaucetInstance

  before(async function () {
    hoprToken = await HoprToken.new()
    hoprFaucet = await HoprFaucet.new(hoprToken.address, false)

    const pauserRole = await hoprFaucet.PAUSER_ROLE()
    const minterRole = await hoprFaucet.MINTER_ROLE()

    await hoprFaucet.grantRole(pauserRole, owner)
    await hoprFaucet.grantRole(minterRole, owner)

    // make HoprFaucet the only minter
    await hoprToken.grantRole(minterRole, hoprFaucet.address, {
      from: owner,
    })
  })

  it('should mint tokens', async function () {
    const receipt = await hoprFaucet.mint(userA, '1', {
      from: userA,
    })

    expectEvent.inTransaction(receipt.tx, HoprToken, 'Transfer', {
      from: constants.ZERO_ADDRESS,
      to: userA,
      value: '1',
    })
  })

  it('should pause minting', async function () {
    await hoprFaucet.pause({
      from: owner,
    })

    expect(await hoprFaucet.paused()).to.be.true

    await expectRevert(
      hoprFaucet.mint(userA, '1', {
        from: userA,
      }),
      'Pausable: paused'
    )
  })

  it('should unpause minting', async function () {
    await hoprFaucet.unpause({
      from: owner,
    })

    expect(await hoprFaucet.paused()).to.be.false

    const receipt = await hoprFaucet.mint(userA, '1', {
      from: userA,
    })

    expectEvent.inTransaction(receipt.tx, HoprToken, 'Transfer', {
      from: constants.ZERO_ADDRESS,
      to: userA,
      value: '1',
    })
  })

  it('should not allow pausing by unauthorized address', async function () {
    await expectRevert(
      hoprFaucet.pause({
        from: userA,
      }),
      'HoprFaucet: caller does not have pauser role'
    )
  })

  it('should not allow unpausing by unauthorized address', async function () {
    await hoprFaucet.pause({
      from: owner,
    })

    await expectRevert(
      hoprFaucet.unpause({
        from: userA,
      }),
      'HoprFaucet: caller does not have pauser role'
    )
  })
})

contract('HoprFaucet with singleUser enabled', function ([owner, userA]) {
  let hoprToken: HoprTokenInstance
  let hoprFaucet: HoprFaucetInstance

  before(async function () {
    hoprToken = await HoprToken.new()
    hoprFaucet = await HoprFaucet.new(hoprToken.address, true)

    const pauserRole = await hoprFaucet.PAUSER_ROLE()
    const minterRole = await hoprFaucet.MINTER_ROLE()

    await hoprFaucet.grantRole(pauserRole, owner)
    await hoprFaucet.grantRole(minterRole, owner)

    // make HoprFaucet the only minter
    await hoprToken.grantRole(minterRole, hoprFaucet.address, {
      from: owner,
    })
  })

  it('should mint tokens', async function () {
    const receipt = await hoprFaucet.mint(owner, '1', {
      from: owner,
    })

    expectEvent.inTransaction(receipt.tx, HoprToken, 'Transfer', {
      from: constants.ZERO_ADDRESS,
      to: owner,
      value: '1',
    })
  })

  it('should pause minting', async function () {
    await hoprFaucet.pause({
      from: owner,
    })

    expect(await hoprFaucet.paused()).to.be.true

    await expectRevert(
      hoprFaucet.mint(userA, '1', {
        from: userA,
      }),
      'Pausable: paused'
    )
  })

  it('should unpause minting', async function () {
    await hoprFaucet.unpause({
      from: owner,
    })

    expect(await hoprFaucet.paused()).to.be.false

    const receipt = await hoprFaucet.mint(owner, '1', {
      from: owner,
    })

    expectEvent.inTransaction(receipt.tx, HoprToken, 'Transfer', {
      from: constants.ZERO_ADDRESS,
      to: owner,
      value: '1',
    })
  })

  it('should not allow minting by unauthorized address', async function () {
    await expectRevert(
      hoprFaucet.mint(userA, '1', {
        from: userA,
      }),
      'HoprFaucet: caller does not have minter role'
    )
  })

  it('should not allow pausing by unauthorized address', async function () {
    await expectRevert(
      hoprFaucet.pause({
        from: userA,
      }),
      'HoprFaucet: caller does not have pauser role'
    )
  })

  it('should not allow unpausing by unauthorized address', async function () {
    await hoprFaucet.pause({
      from: owner,
    })

    await expectRevert(
      hoprFaucet.unpause({
        from: userA,
      }),
      'HoprFaucet: caller does not have pauser role'
    )
  })
})
