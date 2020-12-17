import { deployments } from 'hardhat'
import { BN, expectRevert } from '@openzeppelin/test-helpers'

const MAX_UINT24 = 16777215
const SafeUint24Mock = artifacts.require('SafeUint24Mock')

const useFixtures = deployments.createFixture(async () => {
  const safeMath = await SafeUint24Mock.new()

  return {
    safeMath
  }
})

describe('SafeMath', function () {
  async function testCommutative(fn, lhs, rhs, expected) {
    expect((await fn(lhs, rhs)).toString()).to.be.equal(expected.toString())
    expect((await fn(rhs, lhs)).toString()).to.be.equal(expected.toString())
  }

  async function testFailsCommutative(fn, lhs, rhs, reason) {
    await expectRevert(fn(lhs, rhs), reason)
    await expectRevert(fn(rhs, lhs), reason)
  }

  describe('add', function () {
    it('adds correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('5678')
      const b = new BN('1234')

      await testCommutative(safeMath.add, a, b, a.add(b))
    })

    it('reverts on addition overflow', async function () {
      const { safeMath } = await useFixtures()

      const a = MAX_UINT24
      const b = new BN('1')

      await testFailsCommutative(safeMath.add, a, b, 'SafeUint24: addition overflow')
    })
  })

  describe('div', function () {
    it('divides correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('5678')
      const b = new BN('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal(a.div(b).toString())
    })

    it('divides zero correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('0')
      const b = new BN('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal('0')
    })

    it('returns complete number result on non-even division', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('7000')
      const b = new BN('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal('1')
    })

    it('reverts on division by zero', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('5678')
      const b = new BN('0')

      await expectRevert(safeMath.div(a, b), 'SafeUint24: division by zero')
    })
  })

  describe('mod', function () {
    describe('modulos correctly', async function () {
      const { safeMath } = await useFixtures()

      it('when the dividend is smaller than the divisor', async function () {
        const a = new BN('284')
        const b = new BN('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is equal to the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = new BN('5678')
        const b = new BN('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is larger than the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = new BN('7000')
        const b = new BN('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is a multiple of the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = new BN('17034') // 17034 == 5678 * 3
        const b = new BN('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })
    })

    it('reverts with a 0 divisor', async function () {
      const { safeMath } = await useFixtures()

      const a = new BN('5678')
      const b = new BN('0')

      await expectRevert(safeMath.mod(a, b), 'SafeUint24: modulo by zero')
    })
  })
})
