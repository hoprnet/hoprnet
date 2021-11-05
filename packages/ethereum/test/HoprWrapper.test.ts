import type { PromiseValue } from '@hoprnet/hopr-utils'
import { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { PermittableToken__factory, HoprToken__factory, HoprWrapper__factory } from '../src/types'
import deployERC1820Registry from '../deploy/01_ERC1820Registry'

const useFixtures = deployments.createFixture(async (hre) => {
  const [deployer, userA] = await ethers.getSigners()
  const network = await ethers.provider.getNetwork()

  // deploy ERC1820Registry required by ERC777 tokens
  await deployERC1820Registry(hre, deployer)

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
    deployer,
    userA,
    xHOPR,
    wxHOPR,
    wrapper
  }
})

describe('HoprWrapper', function () {
  let f: PromiseValue<ReturnType<typeof useFixtures>>

  before(async function () {
    f = await useFixtures()
  })

  it('should wrap 10 xHOPR', async function () {
    await expect(f.xHOPR.connect(f.userA).transferAndCall(f.wrapper.address, 10, ethers.constants.HashZero))
      .to.emit(f.wrapper, 'Wrapped')
      .withArgs(f.userA.address, '10')
    expect(await f.xHOPR.balanceOf(f.userA.address)).to.equal('90')
    expect(await f.xHOPR.balanceOf(f.wrapper.address)).to.equal('10')
    expect(await f.wrapper.xHoprAmount()).to.equal('10')
    expect(await f.wxHOPR.balanceOf(f.userA.address)).to.equal('10')
    expect(await f.wxHOPR.totalSupply()).to.equal('10')
  })

  it('should unwrap 10 xHOPR', async function () {
    await expect(f.wxHOPR.connect(f.userA).transfer(f.wrapper.address, 10))
      .to.emit(f.wrapper, 'Unwrapped')
      .withArgs(f.userA.address, '10')
    expect(await f.xHOPR.balanceOf(f.userA.address)).to.equal('100')
    expect(await f.xHOPR.balanceOf(f.wrapper.address)).to.equal('0')
    expect(await f.wrapper.xHoprAmount()).to.equal('0')
    expect(await f.wxHOPR.balanceOf(f.userA.address)).to.equal('0')
    expect(await f.wxHOPR.totalSupply()).to.equal('0')
  })

  it('should not wrap 5 xHOPR when using "transfer"', async function () {
    await expect(f.xHOPR.connect(f.userA).transfer(f.wrapper.address, 5)).to.not.emit(f.wrapper, 'Wrapped')
    expect(await f.xHOPR.balanceOf(f.userA.address)).to.equal('95')
    expect(await f.xHOPR.balanceOf(f.wrapper.address)).to.equal('5')
    expect(await f.wrapper.xHoprAmount()).to.equal('0')
    expect(await f.wxHOPR.balanceOf(f.userA.address)).to.equal('0')
    expect(await f.wxHOPR.totalSupply()).to.equal('0')
  })

  it('should recover 5 xHOPR', async function () {
    await f.wrapper.recoverTokens()
    expect(await f.xHOPR.balanceOf(f.deployer.address)).to.equal('5')
    expect(await f.xHOPR.balanceOf(f.wrapper.address)).to.equal('0')
    expect(await f.wrapper.xHoprAmount()).to.equal('0')
    expect(await f.wxHOPR.totalSupply()).to.equal('0')
  })

  it('should fail when sending an unknown "xHOPR" token', async function () {
    const PermittableToken = (await ethers.getContractFactory('PermittableToken')) as PermittableToken__factory
    const token = await PermittableToken.deploy('Unknown Token', '?', 18, (await ethers.provider.getNetwork()).chainId)
    await token.mint(f.userA.address, 100)

    await expect(token.connect(f.userA).transferAndCall(f.wrapper.address, 10, ethers.constants.HashZero)).to.be
      .reverted
  })

  it('should fail when sending an unknown "wxHOPR" token', async function () {
    const Token = (await ethers.getContractFactory('HoprToken')) as HoprToken__factory
    const token = await Token.deploy()
    await token.grantRole(await token.MINTER_ROLE(), f.deployer.address)
    await token.mint(f.userA.address, 100, ethers.constants.HashZero, ethers.constants.HashZero)

    await expect(token.connect(f.userA).transfer(f.wrapper.address, 10)).to.be.revertedWith('Sender must be wxHOPR')
  })
})
