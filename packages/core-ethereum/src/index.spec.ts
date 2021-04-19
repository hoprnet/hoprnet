import assert from 'assert'
import { stringToU8a, durations } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund, getContracts } from '@hoprnet/hopr-ethereum'
import { ethers } from 'ethers'
import HoprEthereum from '.'
import { createNode } from './utils/testing'
import * as fixtures from './fixtures'
import { providers } from 'ethers'
import { HoprToken__factory, HoprToken } from './contracts'
import { DEFAULT_URI } from './constants'

const { arrayify } = ethers.utils

describe('test connector', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let ownerWallet: ethers.Wallet
  let connector: HoprEthereum

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getContracts().localhost.HoprToken.address} --accounts-to-fund 2`)

    ownerWallet = new ethers.Wallet(fixtures.ACCOUNT_A.privateKey)
    connector = await createNode(arrayify(ownerWallet.privateKey))

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should catch initOnchainValues', async function () {
    this.timeout(10e3)

    const connector = await createNode(stringToU8a(fixtures.ACCOUNT_B.privateKey))

    try {
      await connector.initOnchainValues()
      assert(true)
    } catch (err) {
      assert(false, err)
    }
  })
})

describe('test withdraw', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let provider: providers.WebSocketProvider
  let hoprToken: HoprToken
  let connector: HoprEthereum
  let alice: ethers.Wallet
  let bob: ethers.Wallet

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getContracts().localhost.HoprToken.address} --accounts-to-fund 2`)

    provider = new providers.WebSocketProvider(DEFAULT_URI)

    alice = new ethers.Wallet(fixtures.ACCOUNT_A.privateKey).connect(provider)
    bob = ethers.Wallet.createRandom().connect(provider)
    hoprToken = HoprToken__factory.connect(getContracts().localhost.HoprToken.address, provider)
    connector = await createNode(ethers.utils.arrayify(alice.privateKey))

    await hoprToken.connect(alice).mint(alice.address, 100, ethers.constants.HashZero, ethers.constants.HashZero, {
      gasLimit: 300e3
    })

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should withdraw 1 wei (ETH)', async function () {
    const txHash = await connector.withdraw('NATIVE', bob.address, '1')
    await provider.waitForTransaction(txHash, 1, 5e3)

    assert(txHash.length > 0, 'no transaction hash received')
  })

  it('should withdraw 1 wei (HOPR)', async function () {
    const txHash = await connector.withdraw('HOPR', bob.address, '1')
    await provider.waitForTransaction(txHash, 1, 5e3)

    assert(txHash.length > 0, 'no transaction hash received')
  })
})
