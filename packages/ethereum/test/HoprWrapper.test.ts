import { expect } from 'chai'
import { singletons, expectRevert, expectEvent } from '@openzeppelin/test-helpers'
import { HoprTokenInstance, HoprWrapperInstance } from '../types'
import { vmErrorMessage } from './utils'

const HoprToken = artifacts.require('HoprToken')
const HoprWrapper = artifacts.require('HoprWrapper')

describe.only('HoprWrapper', function () {
  let xHOPR: HoprTokenInstance
  let wxHOPR: HoprTokenInstance
  let wrapper: HoprWrapperInstance
  let deployer: string
  let userA: string

  // @TODO: use fixtures when we merge with upcoming refactor
  before(async function () {
    ;[deployer, userA] = await web3.eth.getAccounts()

    // deploy ERC1820Registry required by ERC777 tokens
    await singletons.ERC1820Registry(deployer)

    // deploy xHOPR
    xHOPR = await HoprToken.new()
    // deploy wxHOPR
    wxHOPR = await HoprToken.new()
    // deploy wrapper
    wrapper = await HoprWrapper.new(xHOPR.address, wxHOPR.address)

    const MINTER_ROLE = await xHOPR.MINTER_ROLE()

    // allow deployer to mint xHOPR tokens for testing
    await xHOPR.grantRole(MINTER_ROLE, deployer)

    // allow wrapper to mint wxHOPR required for swapping
    await wxHOPR.grantRole(MINTER_ROLE, wrapper.address)

    // mint some initial xHOPR for userA
    await xHOPR.mint(userA, 100, '0x0', '0x0')
  })

  it('should wrap 10 xHOPR', async function () {
    const response = await xHOPR.transfer(wrapper.address, 10, {
      from: userA
    })

    await expectEvent.inTransaction(response.tx, wrapper, 'Wrapped', {
      account: userA,
      amount: '10'
    })

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('90')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('10')

    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('10')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('10')
  })

  it('should unwrap 10 xHOPR', async function () {
    const response = await wxHOPR.transfer(wrapper.address, 10, {
      from: userA
    })

    await expectEvent.inTransaction(response.tx, wrapper, 'Unwrapped', {
      account: userA,
      amount: '10'
    })

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('100')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('0')

    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('0')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('0')
  })

  it('should fail when sending an unknown token', async function () {
    const token = await HoprToken.new()
    await token.grantRole(await xHOPR.MINTER_ROLE(), deployer)
    await token.mint(userA, 100, '0x0', '0x0')

    await expectRevert(
      token.transfer(wrapper.address, 10, {
        from: userA
      }),
      vmErrorMessage('Invalid token')
    )
  })
})
