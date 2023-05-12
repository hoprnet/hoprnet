import { Hash, stringToU8a } from '@hoprnet/hopr-utils'
import { peerIdFromString } from '@libp2p/peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import {
  Address,
  Balance,
  BalanceType,
  PublicKey,
  U256,
  Ticket,
  ChannelEntry,
  ChannelStatus
} from '@hoprnet/hopr-utils'
import { validateUnacknowledgedTicket } from './index.js'
import { PeerId } from '@libp2p/interface-peer-id'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET = peerIdFromString('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const TARGET_PUBKEY = () => PublicKey.from_peerid_str(TARGET.toString())
const TARGET_ADDRESS = () => new Address(stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356'))
const SENDER = peerIdFromString('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
// const SENDER_ADDRESS = new Address(stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0'))

const createMockTicket = ({
  sender = SENDER,
  targetAddress = TARGET_ADDRESS(),
  amount = new Balance('1', BalanceType.HOPR),
  win_prob = U256.from_inverse_probability(U256.one()),
  epoch = U256.one(),
  index = U256.one(),
  channel_epoch = U256.one()
}: {
  sender?: PeerId
  targetAddress?: Address
  amount?: Balance
  win_prob?: U256
  epoch?: U256
  index?: U256
  channel_epoch?: U256
}) => {
  return {
    counterparty: targetAddress,
    challenge: new Uint8Array(),
    amount,
    win_prob,
    epoch,
    index,
    channel_epoch,
    verify: (pubKey: PublicKey) => pubKey.eq(PublicKey.from_peerid_str(sender.toString()))
  } as unknown as Ticket
}

const mockChannelEntry = (
  isChannelOpen: boolean = true,
  balance: Balance = new Balance('100', BalanceType.HOPR),
  ticketEpoch = U256.one(),
  ticketIndex = U256.zero()
) =>
  Promise.resolve(
    new ChannelEntry(
      TARGET_PUBKEY(),
      TARGET_PUBKEY(),
      balance,
      Hash.create([stringToU8a('0xdeadbeef')]),
      ticketEpoch,
      ticketIndex,
      isChannelOpen ? ChannelStatus.Open : ChannelStatus.Closed,
      U256.one(),
      U256.zero()
    )
  )

const getTicketsMock = async (): Promise<Ticket[]> => []

describe('messages/validations.spec.ts - unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        getTicketsMock,
        true
      )
    ).to.not.eventually.rejected
  })

  it('should throw when signer is not sender', async function () {
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        TARGET,
        new Balance('2', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith('The signer of the ticket does not match the sender')
  })

  it('should throw when ticket amount is low', async function () {
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('2', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith('Ticket amount')
  })

  it('should throw when ticket chance is low', async function () {
    const signedTicket = createMockTicket({
      win_prob: U256.from_inverse_probability(new U256('2'))
    })

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith('Ticket winning probability')
  })

  it('should throw if there no channel open', async function () {
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(false),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith('is not open')
  })

  it('should throw if ticket epoch does not match our account epoch', async function () {
    const signedTicket = createMockTicket({})
    const mockChannel = await mockChannelEntry(true, new Balance('100', BalanceType.HOPR), new U256('2'))

    return expect(
      validateUnacknowledgedTicket(SENDER, new Balance('1', BalanceType.HOPR), U256.one(), signedTicket, mockChannel, getTicketsMock, true)
    ).to.eventually.rejectedWith('does not match our account epoch')
  })

  it("should throw if ticket's channel iteration does not match the current channel iteration", async function () {
    const signedTicket = createMockTicket({
      channel_epoch: new U256('2')
    })

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith('Ticket was created for a different channel iteration')
  })

  it("should not throw if ticket's index is smaller than the last ticket index", async function () {
    const signedTicket = createMockTicket({})
    const mockChannel = await mockChannelEntry(true, new Balance('100', BalanceType.HOPR), new U256('1'), new U256('2'))

    return expect(
      validateUnacknowledgedTicket(SENDER, new Balance('1', BalanceType.HOPR), U256.one(), signedTicket, mockChannel, getTicketsMock, true)
    ).to.not.eventually.rejected
  })

  it("should not throw if ticket's index is smaller than the last ticket index when you include unredeemed tickets", async function () {
    const signedTicket = createMockTicket({})
    const mockChannel = await mockChannelEntry(true, new Balance('200', BalanceType.HOPR), U256.one(), U256.one())
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance('100', BalanceType.HOPR),
        index: new U256('2')
      })
    ]

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        mockChannel,
        async () => ticketsInDb,
        true
      )
    ).to.not.eventually.rejected
  })

  it('should throw if channel does not have enough funds', async function () {
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(true, Balance.zero(BalanceType.HOPR)),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith(
      'Payment channel 434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df does not have enough funds'
    )
  })

  it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
    const signedTicket = createMockTicket({})
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance('100', BalanceType.HOPR)
      })
    ]

    return expect(
      validateUnacknowledgedTicket(
        SENDER,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        await mockChannelEntry(),
        async () => ticketsInDb,
        true
      )
    ).to.eventually.rejectedWith(
      'Payment channel 434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df does not have enough funds'
    )
  })
})
