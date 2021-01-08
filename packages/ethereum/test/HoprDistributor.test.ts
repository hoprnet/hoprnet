import { expect } from 'chai'
import { BN, singletons, time, expectRevert } from '@openzeppelin/test-helpers'
import { durations } from '@hoprnet/hopr-utils'
import { web3 } from 'hardhat'
import { HoprTokenInstance, HoprDistributorInstance } from '../types'

const HoprToken = artifacts.require('HoprToken')
const HoprDistributor = artifacts.require('HoprDistributor')

const SCHEDULE_UNSET = 'SCHEDULE_UNSET'
const SCHEDULE_1_MIN_ALL = 'SCHEDULE_1_MIN_ALL'
const SCHEDULE_TEAM = 'SCHEDULE_TEAM'

describe('HoprDistributor', function () {
  let owner: string
  let userA: string
  let token: HoprTokenInstance
  let distributor: HoprDistributorInstance
  let multiplier: number

  const getLatestBlockTimestamp = async () => {
    return web3.eth.getBlock(await web3.eth.getBlockNumber()).then((res) => String(res.timestamp))
  }

  const toSolPercent = (percent: number): string => {
    return String(Math.floor(percent * multiplier))
  }

  before(async function () {
    ;[owner, userA] = await web3.eth.getAccounts()
    await singletons.ERC1820Registry(owner)
  })

  // @TODO: use fixture when we merge refactor
  const reset = async (startTime?: string, maxMintAmount?: string) => {
    token = await HoprToken.new()
    distributor = await HoprDistributor.new(
      token.address,
      startTime ?? (await getLatestBlockTimestamp()),
      maxMintAmount ?? '500'
    )
    multiplier = (await distributor.MULTIPLIER()).toNumber()

    await token.grantRole(await token.MINTER_ROLE(), distributor.address, {
      from: owner
    })
  }

  describe('schedules', function () {
    before(async function () {
      await reset()
    })

    it('should add schedule', async function () {
      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
    })

    it('should fail to add schedule again', async function () {
      await expectRevert(distributor.addSchedule([], [], SCHEDULE_1_MIN_ALL), 'Schedule must not exist')
    })

    it('should fail to add schedule with mismatching inputs', async function () {
      await expectRevert(
        distributor.addSchedule(['1'], [], SCHEDULE_UNSET),
        'Durations and percents must have equal length'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expectRevert(
        distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET),
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expectRevert(
        distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET),
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when percent is higher than multiplier', async function () {
      await expectRevert(
        distributor.addSchedule(['1', '1'], ['50', String(Number(multiplier) + 1)], SCHEDULE_UNSET),
        'Percent provided must be smaller or equal to MULTIPLIER'
      )
    })
  })

  describe('allocations', function () {
    before(async function () {
      await reset()
    })

    it('should fail to add allocation when schedule does not exist', async function () {
      await expectRevert(distributor.addAllocations([], [], SCHEDULE_1_MIN_ALL), 'Schedule must exist')
    })

    it('should add allocation', async function () {
      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL)

      expect((await distributor.getClaimable.call(owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
    })

    it('should fail to add allocation with mismatching inputs', async function () {
      await expectRevert(
        distributor.addAllocations([owner], [], SCHEDULE_1_MIN_ALL),
        'Accounts and amounts must have equal length'
      )
    })

    it('should fail to add allocation again', async function () {
      await expectRevert(distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL), 'Allocation must not exist')
    })
  })

  describe('claimable', function () {
    before(async function () {
      await reset()

      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16)
        ],
        [
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7)
        ],
        SCHEDULE_TEAM
      )

      await distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['100'], SCHEDULE_TEAM)
    })

    it('should be able to claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await time.increase(durations.minutes(2))
      expect((await distributor.getClaimable.call(owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('100')
    })

    it('should be able to claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
    })

    it('should be able to claim 14 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('14')
    })

    it('should be able to claim 28 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('28')
    })

    it('should be able to claim 100 after 17 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(10))
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('100')
    })
  })

  describe('claim', function () {
    before(async function () {
      await reset()

      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16)
        ],
        [
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7)
        ],
        SCHEDULE_TEAM
      )

      await distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['100'], SCHEDULE_TEAM)
    })

    it('should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await time.increase(durations.minutes(2))

      await distributor.claim(SCHEDULE_1_MIN_ALL)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('100')
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await distributor.claim(SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('100')
    })

    it('should claim 14 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      await distributor.claim(SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('114')
    })

    it('should claim 28 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      await distributor.claim(SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('128')
    })

    it('should claim 100 after 17 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(10))
      await distributor.claim(SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('200')
    })
  })

  describe('claimFor', function () {
    before(async function () {
      await reset()

      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16)
        ],
        [
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7),
          toSolPercent(1 / 7)
        ],
        SCHEDULE_TEAM
      )

      await distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['100'], SCHEDULE_TEAM)
    })

    it('should claim 100 after 2 minutes using SCHEDULE_1_MIN_ALL', async function () {
      await time.increase(durations.minutes(2))

      await distributor.claimFor(owner, SCHEDULE_1_MIN_ALL)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('100')
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await distributor.claimFor(owner, SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('100')
    })

    it('should claim 14 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      await distributor.claimFor(owner, SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('114')
    })

    it('should claim 28 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      await distributor.claimFor(owner, SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('128')
    })

    it('should claim 100 after 17 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(10))
      await distributor.claimFor(owner, SCHEDULE_TEAM)
      expect((await distributor.getClaimable.call(owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await token.balanceOf.call(owner)).toString()).to.equal('200')
    })
  })

  describe('revoke', function () {
    before(async function () {
      await reset()

      await distributor.addSchedule([durations.minutes(1)], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['100'], SCHEDULE_1_MIN_ALL)
    })

    it('should fail to claim after revoked', async function () {
      await distributor.revokeAccount(owner, SCHEDULE_1_MIN_ALL)
      await expectRevert(distributor.claim(SCHEDULE_1_MIN_ALL), 'Account is revoked')
    })

    it('should fail to revoke if allocation does not exist', async function () {
      await expectRevert(distributor.revokeAccount(owner, SCHEDULE_UNSET), 'Allocation must exist')
    })
  })

  describe('max mint', function () {
    before(async function () {
      await reset(undefined, '50')

      await distributor.addSchedule([0], [toSolPercent(1)], SCHEDULE_1_MIN_ALL)
      await distributor.addAllocations([owner], ['51'], SCHEDULE_1_MIN_ALL)
    })

    it('should fail to claim if max mint is reached', async function () {
      await expectRevert.assertion(distributor.claim(SCHEDULE_1_MIN_ALL))
    })
  })
})
