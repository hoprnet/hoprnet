import { expect } from 'chai'
import { deployments } from 'hardhat'
import { singletons, expectRevert, expectEvent } from '@openzeppelin/test-helpers'
import { PermittableTokenInstance, HoprTokenInstance, HoprWrapperInstance } from '../types'
import { vmErrorMessage } from './utils'

const PermittableToken = artifacts.require('PermittableToken')
const HoprToken = artifacts.require('HoprToken')
const HoprWrapper = artifacts.require('HoprWrapper')

const useFixtures = deployments.createFixture(async () => {
  const [deployer, userA] = await web3.eth.getAccounts()

  // deploy ERC1820Registry required by ERC777 tokens
  await singletons.ERC1820Registry(deployer)

  // deploy xHOPR
  const xHOPR = await PermittableToken.new('xHOPR Token', 'xHOPR', 18, await web3.eth.getChainId())
  // deploy wxHOPR
  const wxHOPR = await HoprToken.new()
  // deploy wrapper
  const wrapper = await HoprWrapper.new(xHOPR.address, wxHOPR.address)

  // allow wrapper to mint wxHOPR required for swapping
  await wxHOPR.grantRole(await wxHOPR.MINTER_ROLE(), wrapper.address)

  // mint some initial xHOPR for userA
  await xHOPR.mint(userA, 100)

  return {
    deployer,
    userA,
    xHOPR,
    wxHOPR,
    wrapper
  }
})

describe('HoprWrapper', function () {
  let xHOPR: PermittableTokenInstance
  let wxHOPR: HoprTokenInstance
  let wrapper: HoprWrapperInstance
  let deployer: string
  let userA: string

  // @TODO: use fixtures when we merge with upcoming refactor
  before(async function () {
    const fixtures = await useFixtures()

    xHOPR = fixtures.xHOPR
    wxHOPR = fixtures.wxHOPR
    wrapper = fixtures.wrapper
    deployer = fixtures.deployer
    userA = fixtures.userA
  })

  it('should wrap 10 xHOPR', async function () {
    const response = await xHOPR.transferAndCall(wrapper.address, 10, '0x0', {
      from: userA
    })

    await expectEvent.inTransaction(response.tx, wrapper, 'Wrapped', {
      account: userA,
      amount: '10'
    })

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('90')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('10')
    expect((await wrapper.xHoprAmount()).toString()).to.equal('10')
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
    expect((await wrapper.xHoprAmount()).toString()).to.equal('0')
    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('0')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('0')
  })

  it('should not wrap 5 xHOPR when using "transfer"', async function () {
    const response = await xHOPR.transfer(wrapper.address, 5, {
      from: userA
    })

    await expectEvent.notEmitted.inTransaction(response.tx, wrapper, 'Wrapped')

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('95')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('5')
    expect((await wrapper.xHoprAmount()).toString()).to.equal('0')
    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('0')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('0')
  })

  it('should recover 5 xHOPR', async function () {
    await wrapper.recoverTokens()

    expect((await xHOPR.balanceOf(deployer)).toString()).to.equal('5')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('0')
    expect((await wrapper.xHoprAmount()).toString()).to.equal('0')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('0')
  })

  it('should fail when sending an unknown "xHOPR" token', async function () {
    const token = await PermittableToken.new('Unknown Token', '?', 18, await web3.eth.getChainId())
    await token.mint(userA, 100)

    await expectRevert.unspecified(
      token.transferAndCall(wrapper.address, 10, '0x0', {
        from: userA
      })
    )
  })

  it('should fail when sending an unknown "wxHOPR" token', async function () {
    const token = await HoprToken.new()
    await token.grantRole(await token.MINTER_ROLE(), deployer)
    await token.mint(userA, 100, '0x0', '0x0')

    await expectRevert(
      token.transfer(wrapper.address, 10, {
        from: userA
      }),
      vmErrorMessage('Sender must be wxHOPR')
    )
  })
})
