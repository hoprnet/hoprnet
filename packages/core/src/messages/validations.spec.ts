import type Hopr from '..'
import { stringToU8a } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import PeerId from 'peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import sinon from 'sinon'
import { Channel } from '@hoprnet/hopr-core-ethereum'
import { Address, Balance, PublicKey, UINT256, Ticket, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import { validateUnacknowledgedTicket, validateCreatedTicket } from '.'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const TARGET_PUBKEY = PublicKey.fromPeerId(TARGET)
const TARGET_ADDRESS = new Address(stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356'))
const SENDER = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
// const SENDER_ADDRESS = new Address(stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0'))

const createMockTicket = ({
  sender = SENDER,
  targetAddress = TARGET_ADDRESS,
  amount = new Balance(new BN(1)),
  winProb = UINT256.fromInverseProbability(new BN(1)),
  epoch = new UINT256(new BN(1)),
  index = new UINT256(new BN(1)),
  channelIteration = new UINT256(new BN(1))
}: {
  sender?: PeerId
  targetAddress?: Address
  amount?: Balance
  winProb?: UINT256
  epoch?: UINT256
  index?: UINT256
  channelIteration?: UINT256
}) => {
  return {
    counterparty: targetAddress,
    challenge: new Uint8Array(),
    amount,
    winProb,
    epoch,
    index,
    channelIteration,
    verify: (pubKey: PublicKey) => pubKey.eq(new PublicKey(sender.pubKey.marshal()))
  } as unknown as Ticket
}

const mockChannelEntry = (isChannelOpen: boolean, balance: Balance, ticketEpoch: UINT256, ticketIndex: UINT256) =>
  Promise.resolve(
    new ChannelEntry(
      TARGET_PUBKEY,
      TARGET_PUBKEY,
      balance,
      null,
      ticketEpoch,
      ticketIndex,
      isChannelOpen ? ChannelStatus.Open : ChannelStatus.Closed,
      new UINT256(new BN(1)),
      null
    )
  )

const createMockChannel = ({
  isChannelOpen = true,
  isChannelStored = true,
  self = new Balance(new BN(0)),
  counterparty = new Balance(new BN(100)),
  ticketEpoch = new UINT256(new BN(1)),
  ticketIndex = new UINT256(new BN(0))
}: {
  isChannelOpen?: boolean
  isChannelStored?: boolean
  self?: Balance
  counterparty?: Balance
  ticketEpoch?: UINT256
  ticketIndex?: UINT256
}) => {
  return {
    usToThem: () => {
      if (isChannelStored) return mockChannelEntry(isChannelOpen, self, ticketEpoch, ticketIndex)
      throw new Error('state not found')
    },
    themToUs: () => {
      if (isChannelStored) return mockChannelEntry(isChannelOpen, counterparty, ticketEpoch, ticketIndex)
      throw new Error('state not found')
    },
    channelEpoch: new BN(1)
  } as unknown as Channel
}

const createMockNode = ({
  // sender = SENDER,
  // senderAddress = SENDER_ADDRESS,
  target = TARGET
}: {
  sender?: PeerId
  senderAddress?: Address
  target?: PeerId
}) => {
  return {
    getId: sinon.stub().returns(target),
    paymentChannels: {
      types: { PublicKey }
    }
  } as unknown as Hopr
}

const getTicketsMock = async (): Promise<Ticket[]> => []

describe('messages/validations.spec.ts - unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        getTicketsMock
      )
    ).to.not.eventually.rejected
  })

  it('should throw when signer is not sender', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        TARGET,
        new BN(2),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('The signer of the ticket does not match the sender')
  })

  it('should throw when ticket amount is low', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(2),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('Ticket amount')
  })

  it('should throw when ticket chance is low', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({
      winProb: UINT256.fromInverseProbability(new BN(2))
    })

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('Ticket winning probability')
  })

  it('should throw if there no channel open', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({
          isChannelOpen: false
        }),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('is not open')
  })

  it('should throw if channel is not stored', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({
          isChannelStored: false
        }),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('Error while validating unacknowledged ticket, state not found')
  })

  it('should throw if ticket epoch does not match our account epoch', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})
    const mockChannel = createMockChannel({ ticketEpoch: new UINT256(new BN(2)) })

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        mockChannel,
        getTicketsMock
      )
    ).to.eventually.rejectedWith('does not match our account epoch')
  })

  it('should throw if ticket index must be higher than last ticket index', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})
    const mockChannel = createMockChannel({ ticketIndex: new UINT256(new BN(1)) })

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        mockChannel,
        getTicketsMock
      )
    ).to.eventually.rejectedWith('must be higher than last ticket index')
  })

  it("should throw if ticket's channel iteration does not match the current channel iteration", async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({
      channelIteration: new UINT256(new BN(2))
    })

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('Ticket was created for a different channel iteration')
  })

  it('should throw if channel does not have enough funds', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({
          self: new Balance(new BN(100)),
          counterparty: new Balance(new BN(0))
        }),
        getTicketsMock
      )
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })

  it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
    const node = createMockNode({})
    const signedTicket = createMockTicket({})
    const ticketsInDb = [
      createMockTicket({
        amount: new Balance(new BN(100))
      })
    ]

    return expect(
      validateUnacknowledgedTicket(
        node.getId(),
        SENDER,
        new BN(1),
        new BN(1),
        signedTicket,
        createMockChannel({}),
        async () => ticketsInDb
      )
    ).to.eventually.rejectedWith('Payment channel does not have enough funds when you include unredeemed tickets')
  })
})

describe('messages/validations.spec.ts unit test validateCreatedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const ticket = createMockTicket({})
    validateCreatedTicket(new BN(1), ticket)
  })
})
