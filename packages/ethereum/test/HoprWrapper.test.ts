import { expect } from 'chai'
import { singletons, expectRevert, expectEvent } from '@openzeppelin/test-helpers'
import { PermittableTokenInstance, HoprTokenInstance, HoprWrapperInstance } from '../types'
import { vmErrorMessage } from './utils'

const PermittableToken = artifacts.require('PermittableToken')
const HoprToken = artifacts.require('HoprToken')
const HoprWrapper = artifacts.require('HoprWrapper')

const deploy_xHOPR = async () => {
  return PermittableToken.new('xHOPR Token', 'xHOPR', 18, await web3.eth.getChainId())
}

describe('HoprWrapper', function () {
  let xHOPR: PermittableTokenInstance
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
    xHOPR = await deploy_xHOPR()
    // deploy wxHOPR
    wxHOPR = await HoprToken.new()
    // deploy wrapper
    wrapper = await HoprWrapper.new(xHOPR.address, wxHOPR.address)

    // allow wrapper to mint wxHOPR required for swapping
    await wxHOPR.grantRole(await wxHOPR.MINTER_ROLE(), wrapper.address)

    // mint some initial xHOPR for userA
    await xHOPR.mint(userA, 100)
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

  it('should fail when sending an unknown "xHOPR" token', async function () {
    const token = await deploy_xHOPR()
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
