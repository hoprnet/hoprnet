import { expect } from 'chai'
import sinon from 'sinon'
import BN from 'bn.js'
import { getBalance, getNativeBalance } from './account'
import { PROVIDER_CACHE_TTL } from './constants'
import Sinon from 'sinon'
import { Balance, NativeBalance } from './types'

describe('test getBalance', function () {
  let clock: Sinon.SinonFakeTimers

  const address: any = {
    toHex: sinon.stub('')
  }
  const getLiveBalance = (value: string) => async () => new Balance(new BN(value))

  before(function () {
    clock = sinon.useFakeTimers()
  })

  after(function () {
    clock.restore()
  })

  it('should get balance but nothing is cached', async function () {
    const result = await getBalance(getLiveBalance('10'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get balance', async function () {
    const result = await getBalance(getLiveBalance('10'), address, false)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get cached balance', async function () {
    const result = await getBalance(getLiveBalance('20'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should not get cached balance', async function () {
    const result = await getBalance(getLiveBalance('20'), address, false)
    expect(result.toBN().toString()).to.equal('20')
  })

  it('should reset cache', async function () {
    clock.tick(PROVIDER_CACHE_TTL + 1)

    const result = await getBalance(getLiveBalance('30'), address, true)
    expect(result.toBN().toString()).to.equal('30')
  })
})

describe('test getNativeBalance', function () {
  let clock: Sinon.SinonFakeTimers

  const address: any = {
    toHex: sinon.stub('')
  }
  const getLiveNativeBalance = (value: string) => async () => new NativeBalance(new BN(value))

  before(function () {
    clock = sinon.useFakeTimers()
  })

  after(function () {
    clock.restore()
  })

  it('should get balance but nothing is cached', async function () {
    const result = await getNativeBalance(getLiveNativeBalance('10'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get balance', async function () {
    const result = await getNativeBalance(getLiveNativeBalance('10'), address, false)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should get cached balance', async function () {
    const result = await getNativeBalance(getLiveNativeBalance('20'), address, true)
    expect(result.toBN().toString()).to.equal('10')
  })

  it('should not get cached balance', async function () {
    const result = await getNativeBalance(getLiveNativeBalance('20'), address, false)
    expect(result.toBN().toString()).to.equal('20')
  })

  it('should reset cache', async function () {
    clock.tick(PROVIDER_CACHE_TTL + 1)

    const result = await getNativeBalance(getLiveNativeBalance('30'), address, true)
    expect(result.toBN().toString()).to.equal('30')
  })
})
