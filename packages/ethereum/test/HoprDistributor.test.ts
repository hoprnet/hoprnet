import type { PromiseValue } from 'type-fest'
import { expect } from 'chai'
import { deployments } from 'hardhat'
import { singletons, time, expectRevert, expectEvent, BN } from '@openzeppelin/test-helpers'
import { durations } from '@hoprnet/hopr-utils'
import { web3 } from 'hardhat'
import { vmErrorMessage } from './utils'

const HoprToken = artifacts.require('HoprToken')
const HoprDistributor = artifacts.require('HoprDistributor')

const SCHEDULE_UNSET = 'SCHEDULE_UNSET'
const SCHEDULE_1_MIN_ALL = 'SCHEDULE_1_MIN_ALL'
const SCHEDULE_TEAM = 'SCHEDULE_TEAM'

const getLatestBlockTimestamp = async (web3: Web3) => {
  return web3.eth.getBlock(await web3.eth.getBlockNumber()).then((res) => String(res.timestamp))
}

const toSolPercent = (multiplier: number, percent: number): string => {
  return String(Math.floor(percent * multiplier))
}

const useFixtures = deployments.createFixture(async (_, ops: { startTime?: string; maxMintAmount?: string } = {}) => {
  const startTime = ops.startTime ?? (await getLatestBlockTimestamp(web3))
  const maxMintAmount = ops.maxMintAmount ?? '500'

  const [owner] = await web3.eth.getAccounts()

  await singletons.ERC1820Registry(owner)

  const token = await HoprToken.new()
  const distributor = await HoprDistributor.new(token.address, startTime, maxMintAmount)
  const multiplier = (await distributor.MULTIPLIER()).toNumber()

  await token.grantRole(await token.MINTER_ROLE(), distributor.address, {
    from: owner
  })

  return {
    owner,
    token,
    distributor,
    multiplier,
    startTime,
    maxMintAmount
  }
})

describe('HoprDistributor', function () {
  describe('start time', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    beforeEach(async function () {
      const startTime = new BN(await getLatestBlockTimestamp(web3)).add(new BN(String(durations.minutes(5)))).toString()
      f = await useFixtures({ startTime })
    })

    it('should update start time', async function () {
      expect((await f.distributor.startTime()).toString()).to.equal(f.startTime)

      await f.distributor.updateStartTime('1')
      expect((await f.distributor.startTime()).toString()).to.equal('1')
    })

    it('should fail to update start time', async function () {
      await time.increase(durations.minutes(10))

      await expectRevert(f.distributor.updateStartTime('1'), vmErrorMessage('Previous start time must not be reached'))
    })
  })

  describe('schedules', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()
    })

    it('should add schedule', async function () {
      const _durations = [durations.minutes(1)]
      const _percents = [toSolPercent(f.multiplier, 1)]

      const response = await f.distributor.addSchedule(_durations, _percents, SCHEDULE_1_MIN_ALL)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')

      expectEvent(response, 'ScheduleAdded', {
        // durations: [new BN(String(_durations))],
        // percents: [new BN(percents)],
        name: SCHEDULE_1_MIN_ALL
      })

      const { 0: scDurations, 1: scPercents } = await f.distributor.getSchedule(SCHEDULE_1_MIN_ALL)
      expect(scDurations[0].toString()).to.equal(String(_durations[0]))
      expect(scPercents[0].toString()).to.equal(_percents[0])
    })

    it('should fail to add schedule again', async function () {
      await expectRevert(f.distributor.addSchedule([], [], SCHEDULE_1_MIN_ALL), 'Schedule must not exist')
    })

    it('should fail to add schedule with mismatching inputs', async function () {
      await expectRevert(
        f.distributor.addSchedule(['1'], [], SCHEDULE_UNSET),
        'Durations and percents must have equal length'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expectRevert(
        f.distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET),
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when durations are not in ascending order', async function () {
      await expectRevert(
        f.distributor.addSchedule(['5', '1'], ['50', '50'], SCHEDULE_UNSET),
        'Durations must be added in ascending order'
      )
    })

    it('should fail to add schedule when percent is higher than multiplier', async function () {
      await expectRevert(
        f.distributor.addSchedule(['1', '2'], ['50', String(Number(f.multiplier) + 1)], SCHEDULE_UNSET),
        'Percent provided must be smaller or equal to MULTIPLIER'
      )
    })

    it('should fail to add schedule when percents do not sum to 100%', async function () {
      await expectRevert(
        f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 0.5)], SCHEDULE_UNSET),
        vmErrorMessage('Percents must sum to MULTIPLIER amount')
      )
    })
  })

  describe('allocations', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()
    })

    it('should fail to add allocation when schedule does not exist', async function () {
      await expectRevert(f.distributor.addAllocations([], [], SCHEDULE_1_MIN_ALL), 'Schedule must exist')
    })

    it('should add allocation', async function () {
      const accounts = [f.owner]
      const amounts = ['100']

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      const response = await f.distributor.addAllocations(accounts, amounts, SCHEDULE_1_MIN_ALL)

      expectEvent(response, 'AllocationAdded', {
        account: accounts[0],
        amount: amounts[0],
        scheduleName: SCHEDULE_1_MIN_ALL
      })

      expect((await f.distributor.totalToBeMinted()).toString()).to.equal(amounts[0])
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
    })

    it('should fail to add allocation with mismatching inputs', async function () {
      await expectRevert(
        f.distributor.addAllocations([f.owner], [], SCHEDULE_1_MIN_ALL),
        'Accounts and amounts must have equal length'
      )
    })

    it('should fail to add allocation again', async function () {
      await expectRevert(
        f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL),
        'Allocation must not exist'
      )
    })

    it('should add second allocation', async function () {
      const accounts = [f.owner]
      const amounts = ['200']

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_TEAM)
      const response = await f.distributor.addAllocations(accounts, amounts, SCHEDULE_TEAM)

      expectEvent(response, 'AllocationAdded', {
        account: accounts[0],
        amount: amounts[0],
        scheduleName: SCHEDULE_TEAM
      })

      expect((await f.distributor.totalToBeMinted()).toString()).to.equal('300')
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
    })
  })

  describe('claimable', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16),
          durations.minutes(18)
        ],
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
      await time.increase(durations.minutes(2))
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('100')
    })

    it('should be able to claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
    })

    it('should be able to claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('12')
    })

    it('should be able to claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('24')
    })

    it('should be able to claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(12))
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('100')
    })
  })

  describe('claim', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16),
          durations.minutes(18)
        ],
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
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claim(SCHEDULE_1_MIN_ALL)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('100')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '100',
        scheduleName: SCHEDULE_1_MIN_ALL
      })
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await f.distributor.claim(SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('100')
    })

    it('should claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claim(SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('112')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '12',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claim(SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('124')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '12',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(12))

      const response = await f.distributor.claim(SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('200')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '76',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should fail to claim when there is nothing to claim', async function () {
      await expectRevert(f.distributor.claim(SCHEDULE_UNSET), 'There is nothing to claim')
    })
  })

  describe('claimFor', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [
          durations.minutes(4),
          durations.minutes(6),
          durations.minutes(8),
          durations.minutes(10),
          durations.minutes(12),
          durations.minutes(14),
          durations.minutes(16),
          durations.minutes(18)
        ],
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
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claimFor(f.owner, SCHEDULE_1_MIN_ALL)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_1_MIN_ALL)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('100')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '100',
        scheduleName: SCHEDULE_1_MIN_ALL
      })
    })

    it('should claim 0 after 2 minutes using SCHEDULE_TEAM', async function () {
      await f.distributor.claimFor(f.owner, SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('100')
    })

    it('should claim 12 after 5 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claimFor(f.owner, SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('112')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '12',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should claim 24 after 8 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(2))

      const response = await f.distributor.claimFor(f.owner, SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('124')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '12',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should claim 100 after 19 minutes using SCHEDULE_TEAM', async function () {
      await time.increase(durations.minutes(12))

      const response = await f.distributor.claimFor(f.owner, SCHEDULE_TEAM)
      expect((await f.distributor.getClaimable.call(f.owner, SCHEDULE_TEAM)).toString()).to.equal('0')
      expect((await f.token.balanceOf.call(f.owner)).toString()).to.equal('200')

      expectEvent(response, 'Claimed', {
        account: f.owner,
        amount: '76',
        scheduleName: SCHEDULE_TEAM
      })
    })

    it('should fail to claim when there is nothing to claim', async function () {
      await expectRevert(f.distributor.claimFor(f.owner, SCHEDULE_UNSET), 'There is nothing to claim')
    })
  })

  describe('revoke', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures()

      await f.distributor.addSchedule([durations.minutes(1)], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
      await f.distributor.addSchedule(
        [durations.minutes(2), durations.minutes(4)],
        [toSolPercent(f.multiplier, 1 / 2), toSolPercent(f.multiplier, 1 / 2)],
        SCHEDULE_TEAM
      )

      await f.distributor.addAllocations([f.owner], ['100'], SCHEDULE_1_MIN_ALL)
      await f.distributor.addAllocations([f.owner], ['200'], SCHEDULE_TEAM)
    })

    it('should fail to claim SCHEDULE_1_MIN_ALL after revoked', async function () {
      await f.distributor.revokeAccount(f.owner, SCHEDULE_1_MIN_ALL)

      expect((await f.distributor.totalToBeMinted()).toString()).to.equal('200')
      await expectRevert(f.distributor.claim(SCHEDULE_1_MIN_ALL), 'Account is revoked')
    })

    it('should fail to claim SCHEDULE_TEAM after revoked', async function () {
      await time.increase(durations.minutes(2))

      await f.distributor.claim(SCHEDULE_TEAM)
      await f.distributor.revokeAccount(f.owner, SCHEDULE_TEAM)
      expect((await f.distributor.totalToBeMinted()).toString()).to.equal('100')
      await expectRevert(f.distributor.claim(SCHEDULE_TEAM), 'Account is revoked')
    })

    it('should fail to revoke twice', async function () {
      await expectRevert(f.distributor.revokeAccount(f.owner, SCHEDULE_TEAM), 'Allocation must not be already revoked')
    })

    it('should fail to revoke if allocation does not exist', async function () {
      await expectRevert(f.distributor.revokeAccount(f.owner, SCHEDULE_UNSET), 'Allocation must exist')
    })
  })

  describe('max mint', function () {
    let f: PromiseValue<ReturnType<typeof useFixtures>>

    before(async function () {
      f = await useFixtures({
        maxMintAmount: '50'
      })

      await f.distributor.addSchedule([1], [toSolPercent(f.multiplier, 1)], SCHEDULE_1_MIN_ALL)
    })

    it('should fail to allocate if totalToBeMinted is higher than max mint', async function () {
      await expectRevert.assertion(f.distributor.addAllocations([f.owner], ['51'], SCHEDULE_1_MIN_ALL))
    })

    // it('should fail to claim if max mint is reached', async function () {
    //   await expectRevert.assertion(distributor.claim(SCHEDULE_1_MIN_ALL))
    // })
  })
})
