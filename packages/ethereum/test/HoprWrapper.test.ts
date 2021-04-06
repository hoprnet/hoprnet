import { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { singletons } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from './utils'
import {
  PermittableToken__factory,
  PermittableToken,
  HoprToken__factory,
  HoprToken,
  HoprWrapper__factory,
  HoprWrapper
} from '../types'

const useFixtures = deployments.createFixture(async () => {
  const [deployer, userA] = await ethers.getSigners()
  const network = await ethers.provider.getNetwork()

  // deploy ERC1820Registry required by ERC777 tokens
  await singletons.ERC1820Registry(deployer)

  // deploy xHOPR
  const xHOPR = await new PermittableToken__factory(deployer).deploy('xHOPR Token', 'xHOPR', 18, network.chainId)
  // deploy wxHOPR
  const wxHOPR = await new HoprToken__factory(deployer).deploy()
  // deploy wrapper
  const wrapper = await new HoprWrapper__factory(deployer).deploy(xHOPR.address, wxHOPR.address)

  // allow wrapper to mint wxHOPR required for swapping
  await wxHOPR.grantRole(await wxHOPR.MINTER_ROLE(), wrapper.address)

  // mint some initial xHOPR for userA
  await xHOPR.mint(userA.address, 100)

  return {
    deployer: deployer.address,
    userA: userA.address,
    xHOPR,
    wxHOPR,
    wrapper
  }
})

describe('HoprWrapper', function () {
  let xHOPR: PermittableToken
  let wxHOPR: HoprToken
  let wrapper: HoprWrapper
  let deployer: string
  let userA: string

  before(async function () {
    const fixtures = await useFixtures()

    xHOPR = fixtures.xHOPR
    wxHOPR = fixtures.wxHOPR
    wrapper = fixtures.wrapper
    deployer = fixtures.deployer
    userA = fixtures.userA
  })

  it('should wrap 10 xHOPR', async function () {
    expect(
      xHOPR.transferAndCall(wrapper.address, 10, '0x0', {
        from: userA
      })
    )
      .to.emit(wrapper, 'Wrapped')
      .withArgs(userA, '10')

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('90')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('10')
    expect((await wrapper.xHoprAmount()).toString()).to.equal('10')
    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('10')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('10')
  })

  it('should unwrap 10 xHOPR', async function () {
    expect(
      wxHOPR.transfer(wrapper.address, 10, {
        from: userA
      })
    )
      .to.emit(wxHOPR, 'Unwrapped')
      .withArgs(userA, '10')

    // await expectEvent.inTransaction(response.tx, wrapper, 'Unwrapped', {
    //   account: userA,
    //   amount: '10'
    // })

    expect((await xHOPR.balanceOf(userA)).toString()).to.equal('100')
    expect((await xHOPR.balanceOf(wrapper.address)).toString()).to.equal('0')
    expect((await wrapper.xHoprAmount()).toString()).to.equal('0')
    expect((await wxHOPR.balanceOf(userA)).toString()).to.equal('0')
    expect((await wxHOPR.totalSupply()).toString()).to.equal('0')
  })

  it('should not wrap 5 xHOPR when using "transfer"', async function () {
    expect(
      xHOPR.transfer(wrapper.address, 5, {
        from: userA
      })
    ).to.not.emit(wrapper, 'Wrapped')
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
    const PermittableToken = (await ethers.getContractFactory('PermittableToken')) as PermittableToken__factory
    const token = await PermittableToken.deploy('Unknown Token', '?', 18, (await ethers.provider.getNetwork()).chainId)
    await token.mint(userA, 100)

    expect(
      token.transferAndCall(wrapper.address, 10, '0x0', {
        from: userA
      })
    ).to.be.reverted
  })

  it('should fail when sending an unknown "wxHOPR" token', async function () {
    const Token = (await ethers.getContractFactory('HoprToken')) as HoprToken__factory
    const token = await Token.deploy()
    await token.grantRole(await token.MINTER_ROLE(), deployer)
    await token.mint(userA, 100, '0x0', '0x0')

    expect(
      token.transfer(wrapper.address, 10, {
        from: userA
      })
    ).to.be.revertedWith(vmErrorMessage('Sender must be wxHOPR'))
  })
})
