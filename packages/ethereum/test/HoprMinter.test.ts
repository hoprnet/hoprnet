import type { AsyncReturnType } from 'type-fest'
import { singletons, time, expectRevert } from '@openzeppelin/test-helpers'
import { HoprMinterContract, HoprMinterInstance, HoprTokenContract, HoprTokenInstance } from '../types'
import { vmErrorMessage } from './utils'

const HoprToken: HoprTokenContract = artifacts.require('HoprToken')
const HoprMinter: HoprMinterContract = artifacts.require('HoprMinter')

const formatAccount = (res: AsyncReturnType<HoprMinterInstance['accounts']>) => ({
  balance: res[0],
  lastClaim: res[1]
})

describe('HoprMinter', function () {
  let owner: string
  let user: string
  let hoprToken: HoprTokenInstance
  let hoprMinter: HoprMinterInstance

  const approximate = web3.utils.toWei('1', 'ether')
  const maxAmount = web3.utils.toWei('100000000', 'ether')
  const duration = time.duration.days(100)

  const reset = async () => {
    ;[owner, user] = await web3.eth.getAccounts()

    await singletons.ERC1820Registry(owner)
    hoprToken = await HoprToken.new()
    hoprMinter = await HoprMinter.new(hoprToken.address, maxAmount, duration)

    // make HoprMinter the only minter
    await hoprToken.grantRole(await hoprToken.MINTER_ROLE(), hoprMinter.address, {
      from: owner
    })
  }

  // reset contracts for every test
  describe('unit tests', function () {
    beforeEach(async function () {
      await reset()
    })

    it("'user' should fail to 'increaseBalance'", async function () {
      await expectRevert.unspecified(
        hoprMinter.increaseBalance(user, '1', {
          from: user
        })
      )
    })

    it("should fail to 'increaseBalance' after deadline", async function () {
      await time.increase(time.duration.years(1))

      await expectRevert(
        hoprMinter.increaseBalance(user, '1', {
          from: owner
        }),
        vmErrorMessage('HoprMinter: deadline passed')
      )
    })

    it("should fail to 'increaseBalance' past maximum", async function () {
      await expectRevert(
        hoprMinter.increaseBalance(user, web3.utils.toBN(maxAmount).add(web3.utils.toBN(1)).toString(), {
          from: owner
        }),
        vmErrorMessage('HoprMinter: maximum allowed tokens to mint reached')
      )
    })
  })

  // reset contracts once
  describe('integration tests', function () {
    before(async function () {
      await reset()
    })

    it('claim 50 HOPR after 50 days', async function () {
      await hoprMinter.increaseBalance(user, web3.utils.toWei('100', 'ether'), {
        from: owner
      })

      await time.increase(time.duration.days(50))

      await hoprMinter.claim({
        from: user
      })

      const minterBalance = await hoprMinter
        .accounts(user)
        .then(formatAccount)
        .then((res) => {
          return Number(res.balance.toString())
        })

      const balance = await hoprToken.balanceOf(user).then((res) => Number(res.toString()))

      expect(minterBalance).to.be.closeTo(
        Number(web3.utils.toWei('50', 'ether')),
        Number(approximate),
        'wrong minter balance'
      )

      expect(balance).to.be.closeTo(Number(web3.utils.toWei('50', 'ether')), Number(approximate), 'wrong balance')
    })

    it('claim 25 HOPR after 25 days', async function () {
      await time.increase(time.duration.days(25))

      await hoprMinter.claimFor(user, {
        from: owner
      })

      const minterBalance = await hoprMinter
        .accounts(user)
        .then(formatAccount)
        .then((res) => {
          return Number(res.balance.toString())
        })

      const balance = await hoprToken.balanceOf(user).then((res) => Number(res.toString()))

      expect(minterBalance).to.be.closeTo(
        Number(web3.utils.toWei('25', 'ether')),
        Number(approximate),
        'wrong minter balance'
      )

      expect(balance).to.be.closeTo(Number(web3.utils.toWei('75', 'ether')), Number(approximate), 'wrong balance')
    })

    it("increase user's minter balance by 75", async function () {
      await hoprMinter.increaseBalance(user, web3.utils.toWei('75', 'ether'), {
        from: owner
      })

      const minterBalance = await hoprMinter
        .accounts(user)
        .then(formatAccount)
        .then((res) => {
          return Number(res.balance.toString())
        })

      expect(minterBalance).to.be.closeTo(
        Number(web3.utils.toWei('100', 'ether')),
        Number(approximate),
        'wrong minter balance'
      )
    })

    it('claim 60 HOPR after 15 days', async function () {
      await time.increase(time.duration.days(15))

      await hoprMinter.claim({
        from: user
      })

      const minterBalance = await hoprMinter
        .accounts(user)
        .then(formatAccount)
        .then((res) => {
          return Number(res.balance.toString())
        })

      const balance = await hoprToken.balanceOf(user).then((res) => Number(res.toString()))

      expect(minterBalance).to.be.closeTo(
        Number(web3.utils.toWei('40', 'ether')),
        Number(approximate),
        'wrong minter balance'
      )

      expect(balance).to.be.closeTo(Number(web3.utils.toWei('135', 'ether')), Number(approximate), 'wrong balance')
    })

    it('claim remaining HOPR after deadline', async function () {
      await time.increase(time.duration.years(1))

      await hoprMinter.claim({
        from: user
      })

      const minterBalance = await hoprMinter
        .accounts(user)
        .then(formatAccount)
        .then((res) => {
          return Number(res.balance.toString())
        })

      const balance = await hoprToken.balanceOf(user).then((res) => Number(res.toString()))

      expect(minterBalance).to.be.closeTo(
        Number(web3.utils.toWei('0', 'ether')),
        Number(approximate),
        'wrong minter balance'
      )

      expect(balance).to.be.closeTo(Number(web3.utils.toWei('175', 'ether')), Number(approximate), 'wrong balance')
    })
  })
})
