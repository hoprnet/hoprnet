import assert from 'assert'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import { Commitment } from './commitment'
import sinon from 'sinon'

// TODO: replace legacy test
describe('test commitment', function () {
  it('constructor', function () {})

  describe('random pre-image', function () {
    let fakeSet, fakeGet, fakeDB, fakeId

    beforeEach(async function () {
      fakeSet = sinon.fake()
      fakeGet = sinon.fake()
      fakeDB = new LevelUp(MemDown())
      fakeId = 'test'
    })

    it('should publish a hashed secret', async function () {
      let cm = new Commitment(fakeSet, fakeGet, fakeDB, fakeId)

      assert(cm.getCurrentCommitment(), 'gives current commitment')

      /*

      let onChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.getAddress().toHex())).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage)
      assert(preImage.hash().eq(onChainHash))

      await (
        await connector.account.sendTransaction(connector.hoprChannels.updateAccountSecret, preImage.toHex())
      ).wait()

      let updatedOnChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.getAddress().toHex())).secret)
      )

      assert(!onChainHash.eq(updatedOnChainHash), `new and old onChainSecret must not be the same`)

      let updatedPreImage = await connector.hashedSecret.findPreImage(updatedOnChainHash)

      assert(!preImage.eq(updatedPreImage), `new and old pre-image must not be the same`)

      assert(updatedPreImage.hash().eq(updatedOnChainHash))
      */
    })
  })

  describe('deterministic debug pre-image', function () {
    it('should publish a hashed secret', async function () {
      /*
      await connector.hashedSecret.initialize()

      let onChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.getAddress().toHex())).secret)
      )

      let preImage = await connector.hashedSecret.findPreImage(onChainHash)

      assert(preImage.hash().eq(onChainHash))

      await (
        await connector.account.sendTransaction(connector.hoprChannels.updateAccountSecret, preImage.toHex())
      ).wait()

      let updatedOnChainHash = new Hash(
        stringToU8a((await connector.hoprChannels.accounts(connector.account.getAddress().toHex())).secret)
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
      */
    })
  })
})
