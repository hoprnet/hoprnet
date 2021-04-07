import { deployments, ethers } from 'hardhat'
import { expect } from 'chai'
import { SafeUint24Mock__factory } from '../../types'

const MAX_UINT24 = 16777215

const useFixtures = deployments.createFixture(async () => {
  const [signer] = await ethers.getSigners()
  const safeMath = await new SafeUint24Mock__factory(signer).deploy()

  return {
    safeMath
  }
})

describe('SafeMath', function () {
  async function testCommutative(fn, lhs, rhs, expected) {
    expect(await fn(lhs, rhs)).to.be.equal(expected)
    expect(await fn(rhs, lhs)).to.be.equal(expected)
  }

  async function testFailsCommutative(fn, lhs, rhs, reason) {
    expect(fn(lhs, rhs)).to.be.revertedWith(reason)
    expect(fn(rhs, lhs)).to.be.revertedWith(reason)
  }

  describe('add', function () {
    it('adds correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('5678')
      const b = ethers.BigNumber.from('1234')

      await testCommutative(safeMath.add, a, b, a.add(b))
    })

    it('reverts on addition overflow', async function () {
      const { safeMath } = await useFixtures()

      const a = MAX_UINT24
      const b = ethers.BigNumber.from('1')

      await testFailsCommutative(safeMath.add, a, b, 'SafeUint24: addition overflow')
    })
  })

  describe('div', function () {
    it('divides correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('5678')
      const b = ethers.BigNumber.from('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal(a.div(b).toString())
    })

    it('divides zero correctly', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('0')
      const b = ethers.BigNumber.from('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal('0')
    })

    it('returns complete number result on non-even division', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('7000')
      const b = ethers.BigNumber.from('5678')

      expect((await safeMath.div(a, b)).toString()).to.be.equal('1')
    })

    it('reverts on division by zero', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('5678')
      const b = ethers.BigNumber.from('0')

      expect(safeMath.div(a, b)).to.be.revertedWith('SafeUint24: division by zero')
    })
  })

  describe('mod', function () {
    describe('modulos correctly', async function () {
      const { safeMath } = await useFixtures()

      it('when the dividend is smaller than the divisor', async function () {
        const a = ethers.BigNumber.from('284')
        const b = ethers.BigNumber.from('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is equal to the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = ethers.BigNumber.from('5678')
        const b = ethers.BigNumber.from('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is larger than the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = ethers.BigNumber.from('7000')
        const b = ethers.BigNumber.from('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })

      it('when the dividend is a multiple of the divisor', async function () {
        const { safeMath } = await useFixtures()

        const a = ethers.BigNumber.from('17034') // 17034 == 5678 * 3
        const b = ethers.BigNumber.from('5678')

        expect((await safeMath.mod(a, b)).toString()).to.be.equal(a.mod(b).toString())
      })
    })

    it('reverts with a 0 divisor', async function () {
      const { safeMath } = await useFixtures()

      const a = ethers.BigNumber.from('5678')
      const b = ethers.BigNumber.from('0')

      expect(safeMath.mod(a, b)).to.be.revertedWith('SafeUint24: modulo by zero')
    })
  })
})
