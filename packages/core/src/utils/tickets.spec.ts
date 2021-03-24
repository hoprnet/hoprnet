import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import { stringToU8a } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import PeerId from 'peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import sinon from 'sinon'
import { validateUnacknowledgedTicket, validateCreatedTicket } from './tickets'
import { Address } from '@hoprnet/hopr-core-ethereum/src/types'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
const TARGET_ADDRESS = stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0')
const SENDER = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const SENDER_ADDRESS = stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356')

const createMockTicket = ({
  targetAddress = new Address(TARGET_ADDRESS),
  amount = new BN(1),
  winProb = new Uint8Array(1),
  epoch = new BN(1),
  channelIteration = new BN(1)
}: {
  targetAddress?: Address
  amount?: BN
  winProb?: Uint8Array
  epoch?: BN
  channelIteration?: BN
}) => {
  return ({
    counterparty: targetAddress,
    challenge: new Uint8Array(),
    amount,
    winProb,
    epoch,
    channelIteration
  } as unknown) as Types.Ticket
}

const createMockSignedTicket = ({
  sender = SENDER,
  targetAddress = new Address(TARGET_ADDRESS),
  amount = new BN(1),
  winProb = new Uint8Array(1),
  channelIteration = new BN(1)
}: {
  sender?: PeerId
  targetAddress?: Address
  amount?: BN
  winProb?: Uint8Array
  channelIteration?: BN
}) => {
  return ({
    ticket: createMockTicket({ targetAddress, amount, winProb, channelIteration }),
    signer: Promise.resolve(sender.pubKey.marshal())
  } as unknown) as Types.SignedTicket
}

const createMockNode = ({
  sender = SENDER,
  senderAddress = SENDER_ADDRESS,
  target = TARGET,
  targetAddress = TARGET_ADDRESS,
  ticketEpoch = new BN(1),
  ticketAmount = 1,
  ticketWinProb = 1,
  isChannelOpen = true,
  isChannelStored = true,
  balance_a = new BN(0),
  balance_b = new BN(100),
  stateCounter = new BN(1),
  getWinProbabilityAsFloat = 1
}: {
  sender?: PeerId
  senderAddress?: Uint8Array
  target?: PeerId
  targetAddress?: Uint8Array
  ticketEpoch?: BN
  ticketAmount?: number
  ticketWinProb?: number
  isChannelOpen?: boolean
  isChannelStored?: boolean
  balance_a?: BN
  balance_b?: BN
  stateCounter?: BN
  getWinProbabilityAsFloat?: number
}) => {
  const pubKeyToAddress = sinon.stub()
  pubKeyToAddress.withArgs(sender.pubKey.marshal()).returns(Promise.resolve(senderAddress))
  pubKeyToAddress.withArgs(target.pubKey.marshal()).returns(Promise.resolve(targetAddress))

  const isPartyA = sinon.stub()
  isPartyA.withArgs(targetAddress, senderAddress).returns(true)
  isPartyA.withArgs(senderAddress, targetAddress).returns(false)

  const stateCounterToIteration = sinon.stub()
  stateCounterToIteration.withArgs(1).returns(1)
  stateCounterToIteration.withArgs(11).returns(2)

  return ({
    getId: sinon.stub().returns(target),
    ticketAmount: ticketAmount,
    ticketWinProb: ticketWinProb,
    paymentChannels: {
      account: {
        address: targetAddress,
        ticketEpoch: Promise.resolve(ticketEpoch)
      },
      utils: {
        isPartyA: isPartyA,
        pubKeyToAddress,
        getWinProbabilityAsFloat: sinon.stub().returns(getWinProbabilityAsFloat),
        stateCounterToIteration
      },
      channel: {
        isOpen: sinon.stub().returns(Promise.resolve(isChannelOpen)),
        create: isChannelStored
          ? sinon.stub().returns({
              stateCounter: Promise.resolve(stateCounter),
              balance_a: Promise.resolve(balance_a),
              balance_b: Promise.resolve(balance_b)
            })
          : sinon.stub().throws()
      }
    }
  } as unknown) as Hopr<Chain>
}

const getTicketsMock = async (): Promise<Types.SignedTicket[]> => []

describe('unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.to.not.rejected
  })

  it('should throw when signer is not sender', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: await PeerId.create(),
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('The signer of the ticket does not match the sender')
  })

  it('should throw when ticket amount is low', async function () {
    const node = createMockNode({
      ticketAmount: 2
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('Ticket amount')
  })

  it('should throw when ticket chance is low', async function () {
    const node = createMockNode({
      getWinProbabilityAsFloat: 0.5
    })
    const signedTicket = createMockSignedTicket({
      winProb: new Uint8Array(0.5)
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('Ticket winning probability')
  })

  it('should throw if there no channel open', async function () {
    const node = createMockNode({
      isChannelOpen: false
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('is not open')
  })

  it('should throw if channel is not stored', async function () {
    const node = createMockNode({
      isChannelStored: false
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('not found')
  })

  it('should throw if ticket epoch does not match our account counter', async function () {
    const node = createMockNode({
      ticketEpoch: new BN(2)
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('does not match our account counter')
  })

  it("should throw if ticket's channel iteration does not match the current channel iteration", async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({
      channelIteration: new BN(2)
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('Ticket was created for a different channel iteration')
  })

  it('should throw if channel does not have enough funds', async function () {
    const node = createMockNode({
      balance_a: new BN(100),
      balance_b: new BN(0)
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: getTicketsMock
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })

  it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})
    const ticketsInDb = [
      createMockSignedTicket({
        amount: new BN(100)
      })
    ]

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        signedTicket,
        getTickets: async () => ticketsInDb
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds when you include unredeemed tickets')
  })
})

describe('unit test validateCreatedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateCreatedTicket({
        myBalance: new BN(1),
        signedTicket
      })
    ).to.eventually.to.not.rejected
  })

  it('should throw when signer is not sender', async function () {
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateCreatedTicket({
        myBalance: new BN(0),
        signedTicket
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })
})
