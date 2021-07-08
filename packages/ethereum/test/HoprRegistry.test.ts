import { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { HoprToken__factory, HoprDistributor__factory, HoprRegistry__factory } from '../types'
import { toSolPercent } from './utils'
import deployERC1820Registry from '../deploy/01_ERC1820Registry'

const getLatestBlockTimestamp = async () => {
  return ethers.provider.getBlock('latest').then((res) => String(res.timestamp))
}

const useFixtures = deployments.createFixture(async (hre) => {
  const [deployer, userA, userB] = await ethers.getSigners()

  // setup smart contracts
  await deployERC1820Registry(hre, deployer)
  const token = await new HoprToken__factory(deployer).deploy()
  const distributor = await new HoprDistributor__factory(deployer).deploy(
    token.address,
    await getLatestBlockTimestamp(),
    '1000'
  )
  const MULTIPLIER = await distributor.MULTIPLIER()
  await token.grantRole(await token.MINTER_ROLE(), distributor.address)
  const registry = await new HoprRegistry__factory(deployer).deploy(distributor.address)

  // add some allocations
  await distributor.addSchedule([1], [toSolPercent(MULTIPLIER.toNumber(), 1)], 'EarlyTokenBuyers')
  await distributor.addSchedule([1], [toSolPercent(MULTIPLIER.toNumber(), 1)], 'TeamAndAdvisors')
  await distributor.addAllocations([userA.address], ['10'], 'EarlyTokenBuyers')
  await distributor.addAllocations([userB.address], ['20'], 'TeamAndAdvisors')

  return {
    deployer,
    userA,
    userB,
    token,
    distributor,
    registry
  }
})

describe('test HoprRegistry', function () {
  it('should setLink & update state', async function () {
    const { userA, userB, registry } = await useFixtures()

    await expect(registry.connect(userA).setLink(userA.address))
      .to.emit(registry, 'LinkCreated')
      .withArgs(userA.address, userA.address)
    await expect(registry.connect(userB).setLink(userB.address))
      .to.emit(registry, 'LinkCreated')
      .withArgs(userB.address, userB.address)

    expect(await registry.registered(0)).to.equal(userA.address)
    expect(await registry.registered(1)).to.equal(userB.address)
    expect(await registry.links(userA.address)).to.equal(userA.address)
    expect(await registry.links(userB.address)).to.equal(userB.address)
    expect(await registry.getLinks()).to.deep.equal([
      [userA.address, userA.address],
      [userB.address, userB.address]
    ])
  })
})
