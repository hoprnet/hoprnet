import { expect } from 'chai'
import { deployments, ethers } from 'hardhat'
import { toSolPercent, increaseTime } from './utils'
import type { HoprToken, HoprDistributor } from '../src/types'
import deployERC1820Registry from '../deploy/01_ERC1820Registry'

const SCHEDULE_UNSET = 'SCHEDULE_UNSET'
const SCHEDULE_1_MIN_ALL = 'SCHEDULE_1_MIN_ALL'
const SCHEDULE_TEAM = 'SCHEDULE_TEAM'

const getLatestBlockTimestamp = async () => {
  return ethers.provider.getBlock('latest').then((res) => String(res.timestamp))
}

const minutes = (min: number): number => {
  let msecs = min * 60 * 1e3
  return msecs
}

const useFixtures = deployments.createFixture(async (hre, ops: { startTime?: string; maxMintAmount?: string } = {}) => {
  const startTime = ops.startTime ?? (await getLatestBlockTimestamp())
  const maxMintAmount = ops.maxMintAmount ?? '500'
  const [owner] = await ethers.getSigners()

  await deployERC1820Registry(hre, owner)

  const token = (await (await ethers.getContractFactory('HoprToken')).deploy()) as HoprToken
  const distributor = (await (
    await ethers.getContractFactory('HoprDistributor')
  ).deploy(token.address, startTime, maxMintAmount)) as HoprDistributor

  await token.grantRole(await token.MINTER_ROLE(), distributor.address)

  const multiplier = (await distributor.MULTIPLIER()).toNumber()

  return {
    owner: owner.address,
    token,
    distributor,
    multiplier,
    startTime,
    maxMintAmount
  }
})

describe('HoprDistributor', async function () {
  describe('start time', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    beforeEach(async function () {
      const blockNumber = ethers.BigNumber.from(await getLatestBlockTimestamp())
      const startTime = blockNumber.add(minutes(5))
      f = await useFixtures({ startTime: startTime.toString() })
    })

    it('should update start time', async function () {
      expect(await f.distributor.startTime()).to.equal(f.startTime)
      await f.distributor.updateStartTime('1')
      expect(await f.distributor.startTime()).to.equal('1')
    })

    it('should fail to update start time', async function () {
      await increaseTime(ethers.provider, minutes(10))
      await expect(f.distributor.updateStartTime('1')).to.be.revertedWith('Previous start time must not be reached')
    })
  })

  describe('schedules', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()
    })

    it('should add schedule', async function () {
      const _durations = [minutes(1)]
      const _percents = [toSolPercent(f.multiplier, 1)]

      await expect(f.distributor.addSchedule(_durations, _percents, SCHEDULE_1_MIN_ALL))
        .to.emit(f.distributor, 'ScheduleAdded')
        .withArgs(_durations, _percents, SCHEDULE_1_MIN_ALL)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_1_MIN_ALL)).to.equal('0')

      const { 0: scDurations, 1: scPercents } = await f.distributor.getSchedule(SCHEDULE_1_MIN_ALL)
      expect(scDurations[0]).to.equal(_durations[0])
      expect(scPercents[0]).to.equal(_percents[0])
    })

    it('should fail to add schedule again', async function () {
      await expect(f.distributor.addSchedule([], [], SCHEDULE_1_MIN_ALL)).to.be.revertedWith('Schedule must not exist')
    })

    it('should fail to add schedule with mismatching inputs', async function () {
      await expect(f.distributor.addSchedule(['1'], [], SCHEDULE_UNSET)).to.revertedWith(
        'Durations and percents must have equal length'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expect(f.distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET)).to.revertedWith(
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expect(f.distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET)).to.revertedWith(
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when percent is higher than multiplier', async function () {
      await expect(
        f.distributor.addSchedule(['1', '2'], ['50', String(Number(f.multiplier) + 1)], SCHEDULE_UNSET)
      ).to.revertedWith('Percent provided must be smaller or equal to MULTIPLIER')
    })

    it('should fail to add schedule when percents do not sum to 100%', async function () {
      await expect(
        f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 0.5)], SCHEDULE_UNSET)
      ).to.revertedWith('Percents must sum to MULTIPLIER amount')
    })
  })

  describe('allocations', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()
    })

    it('should fail to add allocation when schedule does not exist', async function () {
      await expect(f.distributor.addAllocations([], [], SCHEDULE_1_MIN_ALL)).to.be.revertedWith('Schedule must exist')
    })

    it('should add allocation', async function () {
      const accounts = [f.owner]
      const amounts = ['100']

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await expect(f.distributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL))
        .to.emit(f.distributor, 'AllocationAdded')
        .withArgs(accounts[0], amounts[0], SCHEDULE_1_MIN_ALL)

      expect(await f.distributor.totalToBeMinted()).to.equal(amounts[0])
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_1_MIN_ALL)).to.equal('0')
    })

    it('should fail to add allocation with mismatching inputs', async function () {
      await expect(f.distributor.addAllocations([f.owner], [], SCHEDULE_1_MIN_ALL)).to.be.revertedWith(
        'Accounts and amounts must have equal length'
      )
    })

    it('should fail to add allocation again', async function () {
      await expect(f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)).to.be.revertedWith(
        'Allocation must not exist'
      )
    })

    it('should add second allocation', async function () {
      const accounts = [f.owner]
      const amounts = ['200']

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_TEAM)

      await expect(f.distributor.addAllocations(accounts, amounts, SCHEDULE_TEAM))
        .to.emit(f.distributor, 'AllocationAdded')
        .withArgs(accounts[0], amounts[0], SCHEDULE_TEAM)
      expect(await f.distributor.totalToBeMinted()).to.equal('300')
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
    })
  })

  describe('claimable', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [minutes(4), minutes(6), minutes(8), minutes(10), minutes(12), minutes(14), minutes(16), minutes(18)],
        [
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8)
        ],
        SCHEDULE_TEAM
      )

      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)
      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_TEAM)
    })

    it('should be able to claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await increaseTime(ethers.provider, minutes(2))
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_1_MIN_ALL)).to.equal('100')
    })

    it('should be able to claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
    })

    it('should be able to claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('12')
    })

    it('should be able to claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('24')
    })

    it('should be able to claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(12))
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('100')
    })
  })

  describe('claim', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [minutes(4), minutes(6), minutes(8), minutes(10), minutes(12), minutes(14), minutes(16), minutes(18)],
        [
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8)
        ],
        SCHEDULE_TEAM
      )

      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)
      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_TEAM)
    })

    it('should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claim(SCHEDULE_1_MIN_ALL))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '100', SCHEDULE_1_MIN_ALL)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_1_MIN_ALL)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('100')
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await f.distributor.claim(SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('100')
    })

    it('should claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claim(SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '12', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('112')
    })

    it('should claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claim(SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '12', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('124')
    })

    it('should claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(12))

      await expect(f.distributor.claim(SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '76', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('200')
    })

    it('should fail to claim when there is nothing to claim', async function () {
      await expect(f.distributor.claim(SCHEDULE_UNSET)).to.be.revertedWith('There is nothing to claim')
    })
  })

  describe('claimFor', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [minutes(4), minutes(6), minutes(8), minutes(10), minutes(12), minutes(14), minutes(16), minutes(18)],
        [
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8),
          toSolPercent(f.multiplier, 1 / 8)
        ],
        SCHEDULE_TEAM
      )

      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)
      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_TEAM)
    })

    it('should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claimFor(f.owner, SCHEDULE_1_MIN_ALL))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '100', SCHEDULE_1_MIN_ALL)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_1_MIN_ALL)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('100')
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await f.distributor.claimFor(f.owner, SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('100')
    })

    it('should claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claimFor(f.owner, SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '12', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('112')
    })

    it('should claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await expect(f.distributor.claimFor(f.owner, SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '12', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('124')
    })

    it('should claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await increaseTime(ethers.provider, minutes(12))

      await expect(f.distributor.claimFor(f.owner, SCHEDULE_TEAM))
        .to.emit(f.distributor, 'Claimed')
        .withArgs(f.owner, '76', SCHEDULE_TEAM)
      expect(await f.distributor.getClaimable(f.owner, SCHEDULE_TEAM)).to.equal('0')
      expect(await f.token.balanceOf(f.owner)).to.equal('200')
    })

    it('should fail to claim when there is nothing to claim', async function () {
      await expect(f.distributor.claimFor(f.owner, SCHEDULE_UNSET)).to.be.revertedWith('There is nothing to claim')
    })
  })

  describe('revoke', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [minutes(2), minutes(4)],
        [toSolPercent(f.multiplier, 1 / 2), toSolPercent(f.multiplier, 1 / 2)],
        SCHEDULE_TEAM
      )

      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)
      await f.distributor.addAllocations([f.owner], ['200'], SCHEDULE_TEAM)
    })

    it('should fail to claim SCHEDULE_1_MIN_ALL after revoked', async function () {
      await f.distributor.revokeAccount(f.owner, SCHEDULE_1_MIN_ALL)

      expect(await f.distributor.totalToBeMinted()).to.equal('200')
      await expect(f.distributor.claim(SCHEDULE_1_MIN_ALL)).to.be.revertedWith('Account is revoked')
    })

    it('should fail to claim SCHEDULE_TEAM after revoked', async function () {
      await increaseTime(ethers.provider, minutes(2))

      await f.distributor.claim(SCHEDULE_TEAM)
      await f.distributor.revokeAccount(f.owner, SCHEDULE_TEAM)
      expect(await f.distributor.totalToBeMinted()).to.equal('100')
      await expect(f.distributor.claim(SCHEDULE_TEAM)).to.be.revertedWith('Account is revoked')
    })

    it('should fail to revoke twice', async function () {
      expect(f.distributor.revokeAccount(f.owner, SCHEDULE_TEAM)).to.be.revertedWith(
        'Allocation must not be already revoked'
      )
    })

    it('should fail to revoke if allocation does not exist', async function () {
      expect(f.distributor.revokeAccount(f.owner, SCHEDULE_UNSET)).to.be.revertedWith('Allocation must exist')
    })
  })

  describe('max mint', function () {
    let f: Awaited<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures({
        maxMintAmount: '50'
      })

      await f.distributor.addSchedule([1], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
    })

    it('should fail to allocate if totalToBeMinted is higher than max mint', async function () {
      await expect(f.distributor.addAllocations([f.owner], ['51'], SCHEDULE_1_MIN_ALL)).to.be.reverted
    })
  })
})
