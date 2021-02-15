import assert from 'assert'
import type HoprEthereum from '.'
import * as DbKeys from './dbKeys'
import * as Utils from './utils'
import * as Types from './types'
import PreImage, { HASHED_SECRET_WIDTH } from './hashedSecret'
import { u8aEquals, durations, stringToU8a } from '@hoprnet/hopr-utils'
import Memdown from 'memdown'
import LevelUp from 'levelup'
import { Ganache } from '@hoprnet/hopr-testing'
import { Network, addresses, abis } from '@hoprnet/hopr-ethereum'
import { migrate, fund } from '@hoprnet/hopr-ethereum'
import Web3 from 'web3'
import type { WebsocketProvider } from 'web3-core'
import * as testconfigs from './config.spec'
import * as configs from './config'
import Account from './account'
import { randomBytes } from 'crypto'
import { hash as hashFunction } from './utils'

const HoprChannelsAbi = abis.HoprChannels

const EMPTY_HASHED_SECRET = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)
const FUND_ARGS = `--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 1`

describe('test hashedSecret', function () {
  this.timeout(durations.minutes(10))
  const ganache = new Ganache()
  let connector: HoprEthereum

  async function generateConnector(debug?: boolean): Promise<HoprEthereum> {
    let web3 = new Web3(configs.DEFAULT_URI)
    const chainId = await Utils.getChainId(web3)
    const network = Utils.getNetworkName(chainId) as Network

    const connector = ({
      hoprChannels: new web3.eth.Contract(HoprChannelsAbi as any, addresses[network].HoprChannels),
      web3,
      db: LevelUp(Memdown()),
      dbKeys: DbKeys,
      utils: Utils,
      types: Types,
      options: {
        debug
      },
      log: () => {}
    } as unknown) as HoprEthereum

    connector.account = new Account(
      connector,
      stringToU8a(testconfigs.DEMO_ACCOUNTS[0]),
      await Utils.privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0])),
      chainId
    )

    connector.hashedSecret = new PreImage(connector.db, connector.account, connector.hoprChannels)

    connector.stop = async () => {
      await connector.account.stop()
      ;(web3.eth.currentProvider as WebsocketProvider).disconnect(1000, 'Stopping HOPR node.')
    }

    return connector
  }

  // const checkIndex = async (index: number, masterSecret: Uint8Array, shouldThrow: boolean) => {
  //   let hash = masterSecret
  //   for (let i = 0; i < index; i++) {
  //     hash = (await connector.utils.hash(hash)).slice(0, HASHED_SECRET_WIDTH)
  //   }

  //   let result,
  //     errThrown = false
  //   try {
  //     result = await connector.hashedSecret.findPreImage(hash)
  //   } catch (err) {
  //     errThrown = true
  //   }

  //   assert(errThrown == shouldThrow, `Must throw an error if, and only if, it is expected.`)

  //   if (shouldThrow) {
  //     assert(errThrown, `Must throw an error`)
  //   } else {
  //     assert(result != null, `Pre-image must have been derivable from the database.`)
  //     assert(
  //       u8aEquals((await connector.utils.hash(result.preImage)).slice(0, HASHED_SECRET_WIDTH), hash) &&
  //         index == result.index + 1
  //     )
  //   }
  // }

  describe('random pre-image', function () {
    this.timeout(durations.minutes(2))

    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(FUND_ARGS)

      connector = await generateConnector()
    })

    after(async function () {
      await connector.stop()
      await ganache.stop()
    })

    it('should publish a hashed secret', async function () {
      await connector.hashedSecret.initialize()

      let onChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(u8aEquals((await hashFunction(preImage.preImage)).slice(0, HASHED_SECRET_WIDTH), onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.account.signTransaction(
            {
              from: (await connector.account.address).toHex(),
              to: connector.hoprChannels.options.address
            },
            connector.hoprChannels.methods.setHashedSecret(new Types.Hash(preImage.preImage).toHex())
          )
        ).send()
      )
      let updatedOnChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      assert(!u8aEquals(onChainHash, updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!u8aEquals(preImage.preImage, updatedPreImage.preImage), `new and old pre-image must not be the same`)

      assert(
        u8aEquals((await hashFunction(updatedPreImage.preImage)).slice(0, HASHED_SECRET_WIDTH), updatedOnChainHash)
      )
    })

    // // Commented due expensive operations
    // it('should generate a hashed secret and recover a pre-Image', async function () {
    //   this.timeout(durations.seconds(22))
    //   await connector.hashedSecret.initialize()

    //   for (let i = 0; i < TOTAL_ITERATIONS / DB_ITERATION_BLOCK_SIZE; i++) {
    //     assert(
    //       (await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(i * DB_ITERATION_BLOCK_SIZE)))) !=
    //         null
    //     )
    //   }

    //   const masterSecret = await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(0)))

    //   await checkIndex(1, masterSecret, false)

    //   await checkIndex(randomInteger(1, TOTAL_ITERATIONS), masterSecret, false)

    //   await checkIndex(TOTAL_ITERATIONS, masterSecret, false)

    //   await checkIndex(0, masterSecret, true)

    //   await checkIndex(TOTAL_ITERATIONS + 1, masterSecret, true)
    // })
  })

  describe('deterministic debug pre-image', function () {
    this.timeout(durations.minutes(2))

    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(FUND_ARGS)

      connector = await generateConnector(true)
    })

    after(async function () {
      await connector.stop()
      await ganache.stop()
    })

    it('should publish a hashed secret', async function () {
      await connector.hashedSecret.initialize()

      let onChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(u8aEquals((await hashFunction(preImage.preImage)).slice(0, HASHED_SECRET_WIDTH), onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.account.signTransaction(
            {
              from: (await connector.account.address).toHex(),
              to: connector.hoprChannels.options.address
            },
            connector.hoprChannels.methods.setHashedSecret(new Types.Hash(preImage.preImage).toHex())
          )
        ).send()
      )

      let updatedOnChainHash = new Types.Hash(
        stringToU8a(
          (await connector.hoprChannels.methods.accounts((await connector.account.address).toHex()).call()).hashedSecret
        )
      )

      assert(!u8aEquals(onChainHash, updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!u8aEquals(preImage.preImage, updatedPreImage.preImage), `new and old pre-image must not be the same`)

      assert(
        u8aEquals((await hashFunction(updatedPreImage.preImage)).slice(0, HASHED_SECRET_WIDTH), updatedOnChainHash)
      )
    })

    it('should reserve a preImage for tickets with 100% winning probabilty resp. should not reserve for 0% winning probability', async function () {
      const firstTicket = new Types.AcknowledgedTicket(connector, undefined, {
        signedTicket: {
          ticket: {
            hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
            winProb: Utils.computeWinningProbability(1)
          }
        } as Types.SignedTicket,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
      })

      assert(await connector.validateTicket(firstTicket), 'ticket with 100% winning probability must always be a win')

      const firstPreImage = new Types.Hash(new Uint8Array(HASHED_SECRET_WIDTH))
      firstPreImage.set(firstTicket.preImage)

      assert(await connector.validateTicket(firstTicket), 'ticket with 100% winning probability must always be a win')

      const secondPreImage = new Types.Hash(new Uint8Array(HASHED_SECRET_WIDTH))
      secondPreImage.set(firstTicket.preImage)

      assert(
        firstPreImage != null &&
          secondPreImage != null &&
          !firstPreImage.eq(secondPreImage) &&
          u8aEquals((await hashFunction(secondPreImage)).slice(0, HASHED_SECRET_WIDTH), firstPreImage)
      )

      const notWinnigTicket = new Types.AcknowledgedTicket(connector, undefined, {
        signedTicket: {
          ticket: {
            hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
            winProb: Utils.computeWinningProbability(0)
          }
        } as Types.SignedTicket,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
      })

      assert(!(await connector.validateTicket(notWinnigTicket)), 'falsy ticket should not be a win')

      assert(await connector.validateTicket(firstTicket), 'ticket with 100% winning probability must always be a win')

      const fourthPreImage = new Types.Hash(new Uint8Array(HASHED_SECRET_WIDTH))
      fourthPreImage.set(firstTicket.preImage)

      assert(
        fourthPreImage != null &&
          !fourthPreImage.eq(secondPreImage) &&
          u8aEquals((await hashFunction(fourthPreImage)).slice(0, HASHED_SECRET_WIDTH), secondPreImage)
      )
    })

    it('should reserve a preImage for tickets with arbitrary winning probability', async function () {
      const ATTEMPTS = 40

      let ticket: Types.AcknowledgedTicket

      for (let i = 0; i < ATTEMPTS; i++) {
        ticket = new Types.AcknowledgedTicket(connector, undefined, {
          signedTicket: {
            ticket: {
              hash: Promise.resolve(new Types.Hash(randomBytes(Types.Hash.SIZE))),
              winProb: Utils.computeWinningProbability(Math.random())
            }
          } as Types.SignedTicket,
          response: new Types.Hash(randomBytes(Types.Hash.SIZE))
        })

        await connector.validateTicket(ticket)

        if (!u8aEquals(ticket.preImage, EMPTY_HASHED_SECRET)) {
          assert(
            await Utils.isWinningTicket(
              await (await ticket.signedTicket).ticket.hash,
              ticket.response,
              ticket.preImage,
              (await ticket.signedTicket).ticket.winProb
            )
          )
        }
      }
    })
  })
})
