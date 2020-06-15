import assert from 'assert'
import Web3 from 'web3'
import { stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import { Ganache, migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprEthereum from '.'
import { HoprToken } from './tsc/web3/HoprToken'
import { Await } from './tsc/utils'
import { cleanupPromiEvent } from './utils'
import { createNode, getPrivKeyData, createAccountAndFund } from './utils/testing'
import * as configs from './config'
import { randomBytes } from 'crypto'

describe('test connector', function () {
  const ganache = new Ganache()
  let owner: Await<ReturnType<typeof getPrivKeyData>>
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum

  before(async function () {
    this.timeout(60e3)

    await ganache.start()
    await migrate()
    await fund(2)

    owner = await getPrivKeyData(stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
    connector = await createNode(owner.privKey)

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  context('nonces', function () {
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
  context('events', function () {
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
            hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() }),
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
})

describe('test connector with 0 ETH and 0 HOPR', function () {
  const ganache = new Ganache()
  let owner: Await<ReturnType<typeof getPrivKeyData>>
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum

  before(async function () {
    this.timeout(60e3)

    await ganache.start()
    await migrate()

    owner = await getPrivKeyData(randomBytes(32))
    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
    connector = await createNode(owner.privKey)

    await connector.start()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  it('should start the connector without any crypto assets', async function () {
    assert(connector.started, 'Connector should have been started')

    assert((await connector.account.balance).isZero(), `HOPR balance must be zero`)

    assert((await connector.account.nativeBalance).isZero(), `ETH balance must be zero`)
  })

  it('should create some 0-valued dummy tickets', async function () {
    const counterparty = await connector.utils.privKeyToPubKey(randomBytes(32))

    const dummyTicket = await connector.channel.createDummyChannelTicket(
      await connector.utils.pubKeyToAccountId(counterparty),
      new connector.types.Hash(randomBytes(32))
    )

    assert(dummyTicket.ticket.getEmbeddedFunds().isZero())

    assert(
      await dummyTicket.verify(connector.account.keys.onChain.pubKey),
      `ticket must be verifyable under the sender's public key`
    )

    assert(u8aEquals(await dummyTicket.signer, connector.account.keys.onChain.pubKey), `signer must be recoverable`)
  })

  // @TODO: move this test to utils
  // context('events', function () {
  //   it('should clear events once resolved', function () {
  //     let numberOfEvents = 0

  //     const once = () => {
  //       return cleanupPromiEvent(hoprToken.events.Transfer(), (event) => {
  //         return new Promise((resolve, reject) => {
  //           event
  //             .on('data', (data) => {
  //               numberOfEvents++
  //               return resolve(data)
  //             })
  //             .on('error', reject)
  //         })
  //       })
  //     }

  //     return new Promise(async (resolve, reject) => {
  //       try {
  //         const receiver = await createAccountAndFund(web3, hoprToken, owner)

  //         await Promise.all([
  //           once(),
  //           hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() }),
  //           hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() }),
  //         ])
  //         await hoprToken.methods.transfer(receiver.address.toHex(), 1).send({ from: owner.address.toHex() })

  //         assert.equal(numberOfEvents, 1, 'check cleanupPromiEvent')
  //         return resolve()
  //       } catch (err) {
  //         return reject(err)
  //       }
  //     })
  //   })
  // })
})
