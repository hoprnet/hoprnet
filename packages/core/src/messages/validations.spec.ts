import { Hash, stringToU8a } from '@hoprnet/hopr-utils'
import { Address, Balance, BalanceType, U256, Ticket, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import { validateUnacknowledgedTicket } from './index.js'
// import { Address as Pack} from '../../lib/core_packet.js'
import assert from 'assert'

// target is party A, sender is party B
const TARGET_ADDRESS = new Address(stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356'))
const SENDER_ADDRESS = new Address(stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0'))

function createMockTicket(ticket?: {
  sender?: Address
  targetAddress?: Address
  amount?: Balance
  win_prob?: U256
  epoch?: U256
  index?: U256
  channel_epoch?: U256
}) {
  ticket = {
    sender: SENDER_ADDRESS.clone(),
    targetAddress: TARGET_ADDRESS.clone(),
    amount: new Balance('1', BalanceType.HOPR),
    win_prob: U256.from_inverse_probability(U256.one()),
    epoch: U256.one(),
    index: U256.one(),
    channel_epoch: U256.one(),
    ...(ticket ?? {})
  }
  return {
    counterparty: ticket.targetAddress,
    challenge: new Uint8Array(),
    amount: ticket.amount,
    win_prob: ticket.win_prob,
    epoch: ticket.epoch,
    index: ticket.index,
    channel_epoch: ticket.channel_epoch,
    verify: (addr: Address) => addr.eq(Address.from_string(ticket.sender.to_string()))
  } as unknown as Ticket
}

function mockChannelEntry(
  isChannelOpen: boolean = true,
  balance: Balance = new Balance('100', BalanceType.HOPR),
  ticketEpoch = U256.one(),
  ticketIndex = U256.zero()
) {
  return new ChannelEntry(
    TARGET_ADDRESS.clone(),
    TARGET_ADDRESS.clone(),
    balance,
    Hash.create([stringToU8a('0xdeadbeef')]),
    ticketEpoch,
    ticketIndex,
    isChannelOpen ? ChannelStatus.Open : ChannelStatus.Closed,
    U256.one(),
    U256.zero()
  )
}

const getTicketsMock = async (): Promise<Ticket[]> => []

describe('messages/validations.spec.ts - unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const signedTicket = createMockTicket()

    await validateUnacknowledgedTicket(
      SENDER_ADDRESS.clone(),
      new Balance('1', BalanceType.HOPR),
      U256.one(),
      signedTicket,
      mockChannelEntry(),
      getTicketsMock,
      true
    )
  })

  it('should throw when signer is not sender', async function () {
    const signedTicket = createMockTicket()

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          TARGET_ADDRESS.clone(),
          new Balance('2', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannelEntry(),
          getTicketsMock,
          true
        ),
      Error('The signer of the ticket does not match the sender')
    )
  })

  it('should throw when ticket amount is low', async function () {
    const signedTicket = createMockTicket()

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('2', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannelEntry(),
          getTicketsMock,
          true
        ),
      Error(`Ticket amount '1' is not equal to '2'`)
    )
  })

  it('should throw when ticket chance is low', async function () {
    const signedTicket = createMockTicket({
      win_prob: U256.from_inverse_probability(new U256('2'))
    })

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('1', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannelEntry(),
          getTicketsMock,
          true
        ),
      Error(
        `Ticket winning probability '57896044618658097711785492504343953926634992332820282019728792003956564819967' is not equal to '115792089237316195423570985008687907853269984665640564039457584007913129639935'`
      )
    )
  })

  it('should throw if there no channel open', async function () {
    const signedTicket = createMockTicket({})

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('1', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannelEntry(false),
          getTicketsMock,
          true
        ),
      Error(`Payment channel with '0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0' is not open or pending to close`)
    )
  })

  it('should throw if ticket epoch does not match our account epoch', async function () {
    const signedTicket = createMockTicket()
    const mockChannel = mockChannelEntry(true, new Balance('100', BalanceType.HOPR), new U256('2'))

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('1', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannel,
          getTicketsMock,
          true
        ),
      Error(
        `Ticket epoch '1' does not match our account epoch 2 of channel 0x434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df`
      )
    )
  })

  it("should throw if ticket's channel iteration does not match the current channel iteration", async function () {
    const signedTicket = createMockTicket({
      channel_epoch: new U256('2')
    })

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('1', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          mockChannelEntry(),
          getTicketsMock,
          true
        ),
      Error(
        `Ticket was created for a different channel iteration 2 != 1 of channel 0x434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df`
      )
    )
  })

  it("should not throw if ticket's index is smaller than the last ticket index", async function () {
    const signedTicket = createMockTicket()
    const mockChannel = mockChannelEntry(true, new Balance('100', BalanceType.HOPR), new U256('1'), new U256('2'))

    await validateUnacknowledgedTicket(
      SENDER_ADDRESS,
      new Balance('1', BalanceType.HOPR),
      U256.one(),
      signedTicket,
      mockChannel,
      getTicketsMock,
      true
    )
  })

  it("should not throw if ticket's index is smaller than the last ticket index when you include unredeemed tickets", async function () {
    const signedTicket = createMockTicket({})
    const mockChannel = mockChannelEntry(true, new Balance('200', BalanceType.HOPR), U256.one(), U256.one())
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance('100', BalanceType.HOPR),
        index: new U256('2')
      })
    ]

    await validateUnacknowledgedTicket(
      SENDER_ADDRESS,
      new Balance('1', BalanceType.HOPR),
      U256.one(),
      signedTicket,
      mockChannel,
      async () => ticketsInDb,
      true
    )
  })

  it('should throw if channel does not have enough funds', async function () {
    const signedTicket = createMockTicket({})

    await assert.rejects(
      async () =>
        validateUnacknowledgedTicket(
          SENDER_ADDRESS,
          new Balance('1', BalanceType.HOPR),
          U256.one(),
          signedTicket,
          await mockChannelEntry(true, Balance.zero(BalanceType.HOPR)),
          getTicketsMock,
          true
        ),
      Error(
        `Payment channel 0x434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df does not have enough funds`
      )
    )
  })

  it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
    const signedTicket = createMockTicket({})
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance('100', BalanceType.HOPR)
      })
    ]

    await assert.rejects(async () =>
      validateUnacknowledgedTicket(
        SENDER_ADDRESS,
        new Balance('1', BalanceType.HOPR),
        U256.one(),
        signedTicket,
        mockChannelEntry(),
        async () => ticketsInDb,
        true
      )
    ),
      Error(
        `Payment channel 0x434c7d4fdeadfc5b67c251d1a421d2d73e90c81355ade7744af5dddf160c27df does not have enough funds`
      )
  })
})
