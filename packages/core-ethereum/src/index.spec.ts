import assert from 'assert'
import Web3 from 'web3'
import { stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { compile, migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/chain/abis/HoprToken.json'
import addresses from '@hoprnet/hopr-ethereum/chain/addresses'
import HoprEthereum from '.'
import { HoprToken } from './tsc/web3/HoprToken'
import { Await } from './tsc/utils'
import { cleanupPromiEvent } from './utils'
import { createNode, getPrivKeyData, createAccountAndFund } from './utils/testing.spec'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { randomBytes } from 'crypto'

describe('test connector', function () {
  const ganache = new Ganache()
  let owner: Await<ReturnType<typeof getPrivKeyData>>
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum

  before(async function () {
    this.timeout(30e3)

    await ganache.start()
    await compile()
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

// we have changed the behaviour of the connector, now it throws when no funds are available
describe.skip('test connector with 0 ETH and 0 HOPR', function () {
  const ganache = new Ganache()
  let owner: Await<ReturnType<typeof getPrivKeyData>>
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum

  before(async function () {
    this.timeout(60e3)

    await ganache.start()
    await compile()
    await migrate()

    owner = await getPrivKeyData(randomBytes(32))
    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.localhost?.HoprToken)
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

describe('test withdraw', function () {
  const ganache = new Ganache()
  let web3: Web3
  let hoprToken: HoprToken
  let connector: HoprEthereum
  let alice: Await<ReturnType<typeof getPrivKeyData>>
  let bob: Await<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(60e3)

    await ganache.start()
    await compile()
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
    // const balance = await web3.eth.getBalance(bob.address.toHex())
    // console.log("balance", balance)

    assert(txHash.length > 0, 'no transaction hash received')
    // assert(balance === "1", "balance is not correct")
  })

  it('should withdraw 1 wei (HOPR)', async function () {
    const txHash = await connector.withdraw('HOPR', bob.address.toHex(), '1')
    // const balance = await hoprToken.methods.balanceOf(bob.address.toHex()).call()
    // console.log("balance", balance)

    assert(txHash.length > 0, 'no transaction hash received')
    // assert(balance === "1", "balance is not correct")
  })
})
