import assert from 'assert'
import { durations, stringToU8a } from '@hoprnet/hopr-utils'
import { Ganache } from '@hoprnet/hopr-testing'
import { getAddresses, migrate, fund } from '@hoprnet/hopr-ethereum'
import HoprEthereum from '.'
import { waitForConfirmation, computeWinningProbability } from './utils'
import { UnacknowledgedTicket, Ticket, Hash } from './types'
import * as testconfigs from './config.spec'
import { createNode } from './utils/testing.spec'
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

      let onChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage)
      assert(preImage.hash().eq(onChainHash))

      await waitForConfirmation(
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
      let updatedOnChainHash = new Hash(
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

      let onChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage.hash().eq(onChainHash))
      await waitForConfirmation(
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

      let updatedOnChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.methods.accounts(connector.account.address.toHex()).call()).secret)
      )

      assert(!onChainHash.eq(updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!preImage.eq(updatedPreImage), `new and old pre-image must not be the same`)

      assert(updatedPreImage.hash().eq(updatedOnChainHash))
    })

    it('should reserve a preImage for tickets with 100% winning probabilty resp. should not reserve for 0% winning probability', async function () {
      const secretA = new Hash(new Uint8Array(Hash.SIZE).fill(0xff))
      const ticket1 = ({
        getHash: () => new Hash(new Uint8Array(Hash.SIZE).fill(0xff)),
        winProb: computeWinningProbability(1)
      } as unknown) as Ticket
      const ut1 = new UnacknowledgedTicket(ticket1, secretA)
      const response1 = new Hash(new Uint8Array(Hash.SIZE).fill(0xff))

      const ack = await connector.account.acknowledge(ut1, response1)

      assert(ack, 'ticket with 100% winning probability must always be a win')
      const ack2 = await connector.account.acknowledge(ut1, response1)
      assert(ack2, 'ticket with 100% winning probability must always be a win')

      assert(
        ack.preImage != null &&
          ack2.preImage != null &&
          !ack.preImage.eq(ack2.preImage) &&
          ack2.preImage.hash().eq(ack.preImage)
      )

      const utfail = new UnacknowledgedTicket(
        ({
          getHash: () => new Hash(new Uint8Array(Hash.SIZE).fill(0xff)),
          winProb: computeWinningProbability(0)
        } as unknown) as Ticket,
        secretA
      )

      const failedAck = await connector.account.acknowledge(utfail, new Hash(new Uint8Array(Hash.SIZE).fill(0xff)))
      assert(failedAck === null, 'falsy ticket should not be a win')

      const ack4 = await connector.account.acknowledge(ut1, response1)
      assert(ack4, 'ticket with 100% winning probability must always be a win')
      assert(ack4.preImage != null && !ack4.preImage.eq(ack2.preImage) && ack4.preImage.hash().eq(ack2.preImage))
    })
  })
})
