import assert from 'assert'
import { durations } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { migrate, fund, getAddresses } from '@hoprnet/hopr-ethereum'
import { ethers } from 'ethers'
import HoprEthereum from '.'
import { createNode } from './utils/testing'
import * as testconfigs from './config.spec'
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
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 2`)

    ownerWallet = new ethers.Wallet(testconfigs.FUND_ACCOUNT_PRIVATE_KEY)
    connector = await createNode(arrayify(ownerWallet.privateKey))

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
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
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 2`)

    provider = new providers.WebSocketProvider(DEFAULT_URI)

    alice = new ethers.Wallet(NODE_SEEDS[0]).connect(provider)
    bob = ethers.Wallet.createRandom().connect(provider)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
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
