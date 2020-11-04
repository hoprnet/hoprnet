import assert from 'assert'
import Web3 from 'web3'
import { stringToU8a, durations } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/lib/chain/abis/HoprToken.json'
import addresses from '@hoprnet/hopr-ethereum/lib/chain/addresses'
import HoprEthereum from '.'
import { HoprToken } from './tsc/web3/HoprToken'
import { Await } from './tsc/utils'
import { cleanupPromiEvent, waitForConfirmationUsingHash } from './utils'
import { createNode, getPrivKeyData, createAccountAndFund } from './utils/testing.spec'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { randomBytes } from 'crypto'

describe('test connector', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let owner: Await<ReturnType<typeof getPrivKeyData>>
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 2`)

    owner = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.localhost?.HoprToken)
    connector = await createNode(owner.privKey)

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  describe('nonces', function () {
    const parallel = 5

    it('should generate nonces in parallel', async function () {
      const latestNonce = await web3.eth.getTransactionCount(owner.address.toHex())
      const results = await Promise.all(
        Array.from({ length: parallel }).map(async (_, expectedNonce) => {
          const nonce = await connector.account.nonce
          return nonce === latestNonce + expectedNonce
        })
      )

      assert(
        results.every((r) => r),
        'incorrect nonces'
      )
    })

    it('should generate next nonce', async function () {
      const latestNonce = await web3.eth.getTransactionCount(owner.address.toHex())
      const nonce = await connector.account.nonce

      assert.equal(nonce, latestNonce + parallel, 'incorrect next nonce')
    })
  })

  // @TODO: move this test to utils
  describe('events', function () {
    it('should clear events once resolved', function () {
      let numberOfEvents = 0

      const once = () => {
        return cleanupPromiEvent(hoprToken.events.Transfer(), (event) => {
          return new Promise((resolve, reject) => {
            event
              .on('data', (data) => {
                numberOfEvents++
                return resolve(data)
              })
              .on('error', reject)
          })
        })
      }

      return new Promise(async (resolve, reject) => {
        try {
          const receiver = await createAccountAndFund(web3, hoprToken, owner)

          await Promise.all([
            once(),
            hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() }),
            hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() })
          ])
          await hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() })

          assert.equal(numberOfEvents, 1, 'check cleanupPromiEvent')
          return resolve()
        } catch (err) {
          return reject(err)
        }
      })
    })
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
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum
  let alice: Await<ReturnType<typeof getPrivKeyData>>
  let bob: Await<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 2`)

    alice = await getPrivKeyData(stringToU8a(NODE_SEEDS[0]))
    bob = await getPrivKeyData(randomBytes(32))

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.localhost?.HoprToken)
    connector = await createNode(alice.privKey)

    await hoprToken.methods.mint(alice.address.toHex(), 100, '0x0', '0x0').send({
      from: alice.address.toHex()
    })

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should withdraw 1 wei (ETH)', async function () {
    const txHash = await connector.withdraw('NATIVE', bob.address.toHex(), '1')
    await waitForConfirmationUsingHash(web3, txHash)

    assert(txHash.length > 0, 'no transaction hash received')
  })

  it('should withdraw 1 wei (HOPR)', async function () {
    const txHash = await connector.withdraw('HOPR', bob.address.toHex(), '1')
    await waitForConfirmationUsingHash(web3, txHash)

    assert(txHash.length > 0, 'no transaction hash received')
  })
})
