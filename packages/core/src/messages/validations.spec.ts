import { peerIdFromString } from '@libp2p/peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import { validateUnacknowledgedTicket } from './index.js'
import {
  Balance,
  BalanceType,
  ChannelEntry,
  Address,
  PublicKey,
  U256,
  Ticket,
  ChannelStatus,
  Hash
} from '../../lib/core_packet.js'
import { stringToU8a } from '@hoprnet/hopr-utils'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET_PRIV = stringToU8a('0x5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca')
const TARGET_PUBKEY = PublicKey.from_privkey(TARGET_PRIV)

const SENDER_PRIV = stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa')
const SENDER_PUBKEY = PublicKey.from_privkey(SENDER_PRIV)

const TARGET = peerIdFromString(TARGET_PUBKEY.to_peerid_str())
//const TARGET_ADDRESS = TARGET_PUBKEY.to_address()

const SENDER = peerIdFromString(SENDER_PUBKEY.to_peerid_str())
//const SENDER_ADDRESS = PARTY_B.public.to_address()

const createMockTicket = ({
                            targetAddress,
                            amount,
                            winProb,
                            epoch,
                            index,
                            channelEpoch
                          }: {
  targetAddress?: Address
  amount?: Balance
  winProb?: U256
  epoch?: U256
  index?: U256
  channelEpoch?: U256
}) => {
  return Ticket.new(
    targetAddress ?? PublicKey.from_privkey(TARGET_PRIV).to_address(),
    undefined,
    epoch ?? U256.one(),
    index ?? U256.one(),
    amount ?? new Balance('1', BalanceType.HOPR),
    winProb ?? U256.from_inverse_probability(U256.one()),
    channelEpoch ?? U256.one(),
    SENDER_PRIV
  )
}


function mockChannelEntry (isChannelOpen?: boolean, // true
  balance?: Balance, // = new Balance('100', BalanceType.HOPR),
  ticketEpoch?: U256, //= U256.one(),
  ticketIndex?: U256// = U256.zero()
) {
  return new ChannelEntry(
    PublicKey.from_privkey(TARGET_PRIV),
    PublicKey.from_privkey(TARGET_PRIV),
    balance ?? new Balance('100', BalanceType.HOPR),
    Hash.create([stringToU8a("0xdeadbeef")]),
    ticketEpoch ?? U256.one(),
    ticketIndex ?? U256.zero(),
    (isChannelOpen ?? true) ? ChannelStatus.Open : ChannelStatus.Closed,
    U256.one(),
    U256.zero()
  )
}

const getTicketsMock = async (): Promise<Ticket[]> => []

describe('messages/validations.spec.ts - unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const signedTicket = createMockTicket({ targetAddress: TARGET_PUBKEY.to_address() })

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
      winProb: U256.from_inverse_probability(U256.from(2))
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
    const mockChannel = await mockChannelEntry(true, new Balance('100', BalanceType.HOPR), U256.from(2))

    return expect(
      validateUnacknowledgedTicket(SENDER, new Balance('1', BalanceType.HOPR), U256.one(), signedTicket, mockChannel, getTicketsMock, true)
    ).to.eventually.rejectedWith('does not match our account epoch')
  })

  it("should throw if ticket's channel iteration does not match the current channel iteration", async function () {
    const signedTicket = createMockTicket({
      channelEpoch: U256.from(2)
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
    const mockChannel = await mockChannelEntry(
      true,
      new Balance('100', BalanceType.HOPR),
      U256.one(),
      U256.from(2)
    )

    return expect(
      validateUnacknowledgedTicket(SENDER, new Balance('1', BalanceType.HOPR), U256.one(), signedTicket, mockChannel, getTicketsMock, true)
    ).to.not.eventually.rejected
  })

  it("should not throw if ticket's index is smaller than the last ticket index when you include unredeemed tickets", async function () {
    const signedTicket = createMockTicket({})
    const mockChannel = await mockChannelEntry(
      true,
      new Balance('200',  BalanceType.HOPR),
      U256.one(),
      U256.one()
    )
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance('100', BalanceType.HOPR),
        index: U256.from(2)
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
        await mockChannelEntry(true, new Balance('0', BalanceType.HOPR)),
        getTicketsMock,
        true
      )
    ).to.eventually.rejectedWith(
      'Payment channel b7049e3285ed36f84f075549cc34190f42f3ce8ed666400d63e928f10f1d5ed8 does not have enough funds'
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
      'Payment channel b7049e3285ed36f84f075549cc34190f42f3ce8ed666400d63e928f10f1d5ed8 does not have enough funds'
    )
  })
})
