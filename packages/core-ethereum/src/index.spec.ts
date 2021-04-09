import assert from 'assert'
import { stringToU8a, durations, PromiseValue } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { migrate, fund, getAddresses } from '@hoprnet/hopr-ethereum'
import HoprEthereum from '.'
import { createNode, getPrivKeyData } from './utils/testing'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { randomBytes } from 'crypto'
import { providers } from 'ethers'
import { HoprToken__factory, HoprToken } from './contracts'

describe('test connector', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let owner: PromiseValue<ReturnType<typeof getPrivKeyData>>
  let connector: HoprEthereum

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 2`)

    owner = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    connector = await createNode(owner.privKey.serialize())

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should catch initOnchainValues', async function () {
    this.timeout(10e3)

    const connector = await createNode(stringToU8a(NODE_SEEDS[NODE_SEEDS.length - 1]))

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
  let provider: providers.JsonRpcProvider
  let hoprToken: HoprToken
  let connector: HoprEthereum
  let alice: PromiseValue<ReturnType<typeof getPrivKeyData>>
  let bob: PromiseValue<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 2`)

    alice = await getPrivKeyData(stringToU8a(NODE_SEEDS[0]))
    bob = await getPrivKeyData(randomBytes(32))

    provider = new providers.JsonRpcProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
    connector = await createNode(alice.privKey.serialize())

    await hoprToken.methods.mint(alice.address.toHex(), 100, '0x0', '0x0').send({
      from: alice.address.toHex(),
      gas: 200e3
    })

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should withdraw 1 wei (ETH)', async function () {
    const txHash = await connector.withdraw('NATIVE', bob.address.toHex(), '1')
    await provider.waitForTransaction(txHash)

    assert(txHash.length > 0, 'no transaction hash received')
  })

  it('should withdraw 1 wei (HOPR)', async function () {
    const txHash = await connector.withdraw('HOPR', bob.address.toHex(), '1')
    await provider.waitForTransaction(txHash)

    assert(txHash.length > 0, 'no transaction hash received')
  })
})
