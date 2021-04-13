import { expect } from 'chai'
import sinon from 'sinon'
import BN from 'bn.js'
import { getBalance, getNativeBalance } from './account'
import { PROVIDER_CACHE_TTL } from './constants'
import Sinon from 'sinon'
import { ethers } from 'ethers'

describe('test getBalance', function () {
  let clock: Sinon.SinonFakeTimers

  const address: any = {
    toHex: sinon.stub('')
  }
  const createHoprTokenMock = (value: string): any => {
    return {
      balanceOf: () => ethers.BigNumber.from(value)
    }
  }

  before(function () {
    clock = sinon.useFakeTimers()
  })

  after(function () {
    clock.restore()
  })

  it('should get balance but nothing is cached', async function () {
    const result = await getBalance(createHoprTokenMock('10'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get balance', async function () {
    const result = await getBalance(createHoprTokenMock('10'), address, false)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get cached balance', async function () {
    const result = await getBalance(createHoprTokenMock('20'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should not get cached balance', async function () {
    const result = await getBalance(createHoprTokenMock('20'), address, false)
    expect(result.toBN().toString()).to.equal('20')
  })

  it('should reset cache', async function () {
    clock.tick(PROVIDER_CACHE_TTL + 1)

    const result = await getBalance(createHoprTokenMock('30'), address, true)
    expect(result.toBN().toString()).to.equal('30')
  })
})

describe('test getNativeBalance', function () {
  let clock: Sinon.SinonFakeTimers

  const address: any = {
    toHex: sinon.stub('')
  }
  const createProvider = (value: string): any => {
    return {
      getBalance: async () => new BN(value)
    }
  }

  before(function () {
    clock = sinon.useFakeTimers()
  })

  after(function () {
    clock.restore()
  })

  it('should get balance but nothing is cached', async function () {
    const result = await getNativeBalance(createProvider('10'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get balance', async function () {
    const result = await getNativeBalance(createProvider('10'), address, false)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get cached balance', async function () {
    const result = await getNativeBalance(createProvider('20'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should not get cached balance', async function () {
    const result = await getNativeBalance(createProvider('20'), address, false)
    expect(result.toBN().toString()).to.equal('20')
  })

  it('should reset cache', async function () {
    clock.tick(PROVIDER_CACHE_TTL + 1)

    const result = await getNativeBalance(createProvider('30'), address, true)
    expect(result.toBN().toString()).to.equal('30')
  })
})
