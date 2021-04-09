import type CoreConnector from '.'
import { expect } from 'chai'
import sinon from 'sinon'
import BN from 'bn.js'
import { getBalance, getNativeBalance } from './account'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, getAddresses } from '@hoprnet/hopr-ethereum'
import { stringToU8a, durations, PromiseValue } from '@hoprnet/hopr-utils'
import { getPrivKeyData, createAccountAndFund, createNode } from './utils/testing'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { PROVIDER_CACHE_TTL } from './constants'
import Sinon from 'sinon'
import { ethers, providers } from 'ethers'
import { HoprToken__factory, HoprToken } from './contracts'

describe('test Account', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let provider: providers.JsonRpcProvider
  let hoprToken: HoprToken
  let coreConnector: CoreConnector
  let funder: PromiseValue<ReturnType<typeof getPrivKeyData>>
  let user: PromiseValue<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()

    provider = new providers.JsonRpcProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
  })

  after(async function () {
    await ganache.stop()
  })

  beforeEach(async function () {
    this.timeout(durations.minutes(1))
    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    user = await createAccountAndFund(provider, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[1])
    coreConnector = await createNode(user.privKey.serialize(), false)

    // wait until it starts
    await coreConnector.start()
    await coreConnector.initOnchainValues()
  })

  afterEach(async function () {
    await coreConnector.stop()
  })
})

describe('test getBalance', function () {
  let clock: Sinon.SinonFakeTimers

  const address: any = {
    toHex: sinon.stub('')
  }
  const createHoprTokenMock = (value: string): any => {
    return {
      balanceOf: () => ({
        call: async () => ethers.BigNumber.from(value)
      })
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
