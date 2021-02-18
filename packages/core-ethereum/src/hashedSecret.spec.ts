import type HoprEthereum from '.'
import assert from 'assert'
import { randomBytes } from 'crypto'
import { u8aEquals, durations, stringToU8a } from '@hoprnet/hopr-utils'
import * as Utils from './utils'
import * as Types from './types'
import { HASHED_SECRET_WIDTH, hashFunction } from './hashedSecret'
import { Ganache } from '@hoprnet/hopr-testing'
import { addresses, migrate, fund } from '@hoprnet/hopr-ethereum'
import { getPrivKeyData, createNode } from './utils/testing.spec'
import * as testconfigs from './config.spec'

const EMPTY_HASHED_SECRET = new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00)

describe('test hashedSecret', function () {
  this.timeout(durations.minutes(10))
  const ganache = new Ganache()
  let connector: HoprEthereum

  const reset = async () => {
    await ganache.stop()

    await ganache.start()
    await migrate()
    await fund(`--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 1`)

    const user = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    connector = await createNode(user.privKey)

    await connector.start()
  }

  before(async function () {
    this.timeout(durations.minutes(1))
    await reset()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  describe('random pre-image', function () {
    this.timeout(durations.minutes(2))

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
  })

  describe('deterministic debug pre-image', function () {
    this.timeout(durations.minutes(2))

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

      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

      const firstPreImage = new Types.Hash(new Uint8Array(HASHED_SECRET_WIDTH))
      firstPreImage.set(firstTicket.preImage)

      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

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

      assert(!(await connector.account.reservePreImageIfIsWinning(notWinnigTicket)), 'falsy ticket should not be a win')

      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

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

        await connector.account.reservePreImageIfIsWinning(ticket)

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
