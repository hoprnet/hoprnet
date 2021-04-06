import assert from 'assert'
import { randomBytes } from 'crypto'
import { durations, stringToU8a } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { getAddresses, migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprEthereum from '.'
import * as Utils from './utils'
import * as Types from './types'
import * as testconfigs from './config.spec'
import { createNode } from './utils/testing.spec'

const EMPTY_HASHED_SECRET = new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0x00))
const FUND_ARGS = `--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 1`

// TODO: replace legacy test
describe('test hashedSecret', function () {
  this.timeout(durations.minutes(10))
  const ganache = new Ganache()
  let connector: HoprEthereum

  // instead of using a half-assed mock we use the connector instance
  // the whole test needs to be rewritten
  async function generateConnector(debug?: boolean): Promise<HoprEthereum> {
    const privKey = stringToU8a(testconfigs.DEMO_ACCOUNTS[0])
    return createNode(privKey, debug, 0)
  }

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
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage.hash().eq(onChainHash))

      await connector.utils.waitForConfirmation(
        (
          await connector.account.signTransaction(
            {
              from: connector.account.address.toHex(),
              to: connector.hoprChannels.options.address
            },
            connector.hoprChannels.methods.updateAccountSecret(preImage.toHex())
          )
        ).send()
      )
      let updatedOnChainHash = new Types.Hash(
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      assert(!onChainHash.eq(updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!preImage.eq(updatedPreImage), `new and old pre-image must not be the same`)

      assert(updatedPreImage.hash().eq(updatedOnChainHash))
    })
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
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage.hash().eq(onChainHash))
      await connector.utils.waitForConfirmation(
        (
          await connector.account.signTransaction(
            {
              from: connector.account.address.toHex(),
              to: connector.hoprChannels.options.address
            },
            connector.hoprChannels.methods.updateAccountSecret(preImage.toHex())
          )
        ).send()
      )

      let updatedOnChainHash = new Types.Hash(
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      assert(!onChainHash.eq(updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!preImage.eq(updatedPreImage), `new and old pre-image must not be the same`)

      assert(updatedPreImage.hash().eq(updatedOnChainHash))
    })

    it('should reserve a preImage for tickets with 100% winning probabilty resp. should not reserve for 0% winning probability', async function () {
      const firstTicket = new Types.AcknowledgedTicket(undefined, {
        signedTicket: ({
          ticket: {
            getHash: () => new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
            winProb: Utils.computeWinningProbability(1)
          }
        } as unknown) as Types.SignedTicket,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
      })

      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

      const firstPreImage = firstTicket.preImage.clone()
      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

      const secondPreImage = firstTicket.preImage.clone()

      assert(
        firstPreImage != null &&
          secondPreImage != null &&
          !firstPreImage.eq(secondPreImage) &&
          secondPreImage.hash().eq(firstPreImage)
      )

      const notWinnigTicket = new Types.AcknowledgedTicket(undefined, {
        signedTicket: ({
          ticket: {
            getHash: () => new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff)),
            winProb: Utils.computeWinningProbability(0)
          }
        } as unknown) as Types.SignedTicket,
        response: new Types.Hash(new Uint8Array(Types.Hash.SIZE).fill(0xff))
      })

      assert(!(await connector.account.reservePreImageIfIsWinning(notWinnigTicket)), 'falsy ticket should not be a win')

      assert(
        await connector.account.reservePreImageIfIsWinning(firstTicket),
        'ticket with 100% winning probability must always be a win'
      )

      const fourthPreImage = firstTicket.preImage.clone()

      assert(fourthPreImage != null && !fourthPreImage.eq(secondPreImage) && fourthPreImage.hash().eq(secondPreImage))
    })

    it('should reserve a preImage for tickets with arbitrary winning probability', async function () {
      const ATTEMPTS = 40

      let ticket: Types.AcknowledgedTicket

      for (let i = 0; i < ATTEMPTS; i++) {
        ticket = new Types.AcknowledgedTicket(undefined, {
          signedTicket: ({
            ticket: {
              getHash: () => new Types.Hash(randomBytes(Types.Hash.SIZE)),
              winProb: Utils.computeWinningProbability(Math.random())
            }
          } as unknown) as Types.SignedTicket,
          response: new Types.Hash(randomBytes(Types.Hash.SIZE))
        })

        await connector.account.reservePreImageIfIsWinning(ticket)

        if (!ticket.preImage.eq(EMPTY_HASHED_SECRET)) {
          assert(
            await Utils.isWinningTicket(
              (await ticket.signedTicket).ticket.getHash(),
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
