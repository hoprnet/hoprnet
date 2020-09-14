import assert from 'assert'
import type HoprEthereum from '.'
import * as DbKeys from './dbKeys'
import * as Utils from './utils'
import * as Types from './types'
import PreImage, { GIANT_STEP_WIDTH, TOTAL_ITERATIONS, HASHED_SECRET_WIDTH } from './hashedSecret'
import { randomInteger, u8aEquals, durations, stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import Memdown from 'memdown'
import LevelUp from 'levelup'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund } from '@hoprnet/hopr-ethereum'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import Web3 from 'web3'
import * as testconfigs from './config.spec'
import * as configs from './config'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import Account from './account'
import { addresses } from '@hoprnet/hopr-ethereum'
import { randomBytes } from 'crypto'

describe('test hashedSecret', function () {
  this.timeout(durations.seconds(7))
  const ganache = new Ganache()
  let connector: HoprEthereum

  async function generateConnector(debug?: boolean): Promise<HoprEthereum> {
    let web3 = new Web3(configs.DEFAULT_URI)
    const chainId = await Utils.getChainId(web3)
    const network = Utils.getNetworkName(chainId) as addresses.Networks

    const connector = ({
      signTransaction: Utils.TransactionSigner(web3, stringToU8a(NODE_SEEDS[0])),
      hoprChannels: new web3.eth.Contract(HoprChannelsAbi as any, configs.CHANNELS_ADDRESSES[network]),
      web3,
      db: LevelUp(Memdown()),
      dbKeys: DbKeys,
      utils: Utils,
      types: Types,
      options: {
        debug,
      },
      log: () => {},
    } as unknown) as HoprEthereum

    connector.account = new Account(
      connector,
      stringToU8a(testconfigs.DEMO_ACCOUNTS[0]),
      await Utils.privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))
    )

    connector.hashedSecret = new PreImage(connector)

    return connector
  }

  const checkIndex = async (index: number, masterSecret: Uint8Array, shouldThrow: boolean) => {
    let hash = masterSecret
    for (let i = 0; i < index; i++) {
      hash = (await connector.utils.hash(hash)).slice(0, HASHED_SECRET_WIDTH)
    }

    let result,
      errThrown = false
    try {
      result = await connector.hashedSecret.findPreImage(hash)
    } catch (err) {
      errThrown = true
    }

    assert(errThrown == shouldThrow, `Must throw an error if, and only if, it is expected.`)

    if (shouldThrow) {
      assert(errThrown, `Must throw an error`)
    } else {
      assert(result != null, `Pre-image must have been derivable from the database.`)
      assert(
        u8aEquals((await connector.utils.hash(result.preImage)).slice(0, HASHED_SECRET_WIDTH), hash) &&
          index == result.index + 1
      )
    }
  }

  context('random pre-image', function () {
    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(1)

      connector = await generateConnector()
    })

    after(async function () {
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

      assert(u8aEquals((await connector.utils.hash(preImage.preImage)).slice(0, HASHED_SECRET_WIDTH), onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.signTransaction(connector.hoprChannels.methods.setHashedSecret(preImage.preImage.toHex()), {
            from: (await connector.account.address).toHex(),
            to: connector.hoprChannels.options.address,
            nonce: await connector.account.nonce,
          })
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
        u8aEquals(
          (await connector.utils.hash(updatedPreImage.preImage)).slice(0, HASHED_SECRET_WIDTH),
          updatedOnChainHash
        )
      )
    })

    // Commented due expensive operations
    it.skip('should generate a hashed secret and recover a pre-Image', async function () {
      this.timeout(durations.seconds(22))
      await connector.hashedSecret.initialize()

      for (let i = 0; i < TOTAL_ITERATIONS / GIANT_STEP_WIDTH; i++) {
        assert(
          (await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(i * GIANT_STEP_WIDTH)))) !=
            null
        )
      }

      const masterSecret = await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(0)))

      await checkIndex(1, masterSecret, false)

      await checkIndex(randomInteger(1, TOTAL_ITERATIONS), masterSecret, false)

      await checkIndex(TOTAL_ITERATIONS, masterSecret, false)

      await checkIndex(0, masterSecret, true)

      await checkIndex(TOTAL_ITERATIONS + 1, masterSecret, true)
    })
  })

  context('deterministic debug pre-image', function () {
    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(1)

      connector = await generateConnector(true)
    })

    after(async function () {
      await connector.account.stop()
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

      assert(u8aEquals((await connector.utils.hash(preImage.preImage)).slice(0, HASHED_SECRET_WIDTH), onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.signTransaction(connector.hoprChannels.methods.setHashedSecret(preImage.preImage.toHex()), {
            from: (await connector.account.address).toHex(),
            to: connector.hoprChannels.options.address,
            nonce: await connector.account.nonce,
          })
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
        u8aEquals(
          (await connector.utils.hash(updatedPreImage.preImage)).slice(0, HASHED_SECRET_WIDTH),
          updatedOnChainHash
        )
      )
    })

    it('should reserve a preImage for tickets with 100% winning probabilty resp. should not reserve for 0% winning probability', async function () {
      const firstTicket = {
        signedTicket: Promise.resolve({
          ticket: {
            hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
            winProb: Utils.computeWinningProbability(1),
          },
        }) as Promise<Types.SignedTicket>,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
      } as Types.AcknowledgedTicket

      const firstPreImage = await connector.hashedSecret.reserveIfIsWinning(firstTicket)

      const secondPreImage = await connector.hashedSecret.reserveIfIsWinning(firstTicket)

      assert(
        firstPreImage != null &&
          secondPreImage != null &&
          !(firstPreImage as Types.Hash).eq(secondPreImage as Types.Hash) &&
          u8aEquals(
            (await Utils.hash(secondPreImage as Types.Hash)).slice(0, HASHED_SECRET_WIDTH),
            firstPreImage as Types.Hash
          )
      )

      const notWinnigTicket = {
        signedTicket: Promise.resolve({
          ticket: {
            hash: Promise.resolve(new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))),
            winProb: Utils.computeWinningProbability(0),
          },
        }) as Promise<Types.SignedTicket>,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
      } as Types.AcknowledgedTicket

      const thirdAttempt = await connector.hashedSecret.reserveIfIsWinning(notWinnigTicket)

      assert(thirdAttempt == null, `Ticket with winning probability of 0 should not be a win`)

      const fourthAttempt = await connector.hashedSecret.reserveIfIsWinning(firstTicket)

      assert(
        fourthAttempt != null &&
          !(fourthAttempt as Types.Hash).eq(secondPreImage as Types.Hash) &&
          u8aEquals(
            (await Utils.hash(fourthAttempt as Types.Hash)).slice(0, HASHED_SECRET_WIDTH),
            secondPreImage as Types.Hash
          )
      )
    })

    it('should reserve a preImage for tickets with arbitrary winning probabiltiy', async function () {
      const ATTEMPTS = 40

      let ticket: Types.AcknowledgedTicket

      let preImage: Types.Hash | void

      for (let i = 0; i < ATTEMPTS; i++) {
        ticket = {
          signedTicket: Promise.resolve({
            ticket: {
              hash: Promise.resolve(new Types.Hash(randomBytes(Types.Hash.SIZE))),
              winProb: Utils.computeWinningProbability(Math.random()),
            },
          }) as Promise<Types.SignedTicket>,
          response: new Types.Hash(randomBytes(Types.Hash.SIZE)),
        } as Types.AcknowledgedTicket

        preImage = await connector.hashedSecret.reserveIfIsWinning(ticket)

        if (preImage != null) {
          assert(
            await Utils.isWinningTicket(
              await (await ticket.signedTicket).ticket.hash,
              ticket.response,
              preImage as Types.Hash,
              (await ticket.signedTicket).ticket.winProb
            )
          )
        }
      }
    })
  })

  context('integration', function () {
    before(async function () {
      this.timeout(durations.minutes(1))
      await ganache.start()
      await migrate()
      await fund(1)

      connector = await generateConnector()
    })

    after(async function () {
      await ganache.stop()
    })

    it('should initialize hashedSecret', async function () {
      assert(!(await connector.hashedSecret.check()).initialized, "hashedSecret shouldn't be initialized")

      await connector.hashedSecret.initialize()
      assert((await connector.hashedSecret.check()).initialized, 'hashedSecret should be initialized')
    })

    it('should already be initialized', async function () {
      await connector.hashedSecret.initialize()
      assert((await connector.hashedSecret.check()).initialized, 'hashedSecret should be initialized')
    })

    it('should reinitialize hashedSecret when off-chain secret is missing', async function () {
      connector.db = LevelUp(Memdown())

      await connector.hashedSecret.initialize()
      assert((await connector.hashedSecret.check()).initialized, 'hashedSecret should be initialized')
    })

    it('should submit hashedSecret when on-chain secret is missing', async function () {
      const db = connector.db

      this.timeout(durations.minutes(1))
      await ganache.stop()
      await ganache.start()
      await migrate()
      await fund(1)

      connector = await generateConnector()
      connector.db = db

      await connector.hashedSecret.initialize()
      assert((await connector.hashedSecret.check()).initialized, 'hashedSecret should be initialized')
    })
  })
})
