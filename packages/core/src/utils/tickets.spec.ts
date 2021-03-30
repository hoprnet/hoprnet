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
import { Address, Balance, Public, Hash, UINT256, Utils } from '@hoprnet/hopr-core-ethereum'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const TARGET_ADDRESS = new Address(stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356'))
const SENDER = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
// const SENDER_ADDRESS = new Address(stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0'))

const createMockTicket = ({
  targetAddress = TARGET_ADDRESS,
  amount = new Balance(new BN(1)),
  winProb = Utils.computeWinningProbability(1),
  epoch = new UINT256(new BN(1)),
  channelIteration = new UINT256(new BN(1))
}: {
  targetAddress?: Address
  amount?: Balance
  winProb?: Hash
  epoch?: UINT256
  channelIteration?: UINT256
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
  targetAddress = TARGET_ADDRESS,
  amount = new Balance(new BN(1)),
  winProb = Utils.computeWinningProbability(1),
  channelIteration = new UINT256(new BN(1))
}: {
  sender?: PeerId
  targetAddress?: Address
  amount?: Balance
  winProb?: Hash
  channelIteration?: UINT256
}) => {
  return ({
    ticket: createMockTicket({ targetAddress, amount, winProb, channelIteration }),
    signer: Promise.resolve(new Public(sender.pubKey.marshal()))
  } as unknown) as Types.SignedTicket
}

const createMockNode = ({
  // sender = SENDER,
  // senderAddress = SENDER_ADDRESS,
  target = TARGET,
  targetAddress = TARGET_ADDRESS,
  ticketEpoch = new UINT256(new BN(1)),
  ticketAmount = 1,
  ticketWinProb = 1,
  isChannelOpen = true,
  isChannelStored = true,
  balance_a = new Balance(new BN(0)),
  balance_b = new Balance(new BN(100)),
  stateCounter = new UINT256(new BN(1))
}: {
  sender?: PeerId
  senderAddress?: Address
  target?: PeerId
  targetAddress?: Address
  ticketEpoch?: UINT256
  ticketAmount?: number
  ticketWinProb?: number
  isChannelOpen?: boolean
  isChannelStored?: boolean
  balance_a?: Balance
  balance_b?: Balance
  stateCounter?: UINT256
}) => {
  return ({
    getId: sinon.stub().returns(target),
    ticketAmount: ticketAmount,
    ticketWinProb: ticketWinProb,
    paymentChannels: {
      account: {
        address: targetAddress,
        ticketEpoch: Promise.resolve(ticketEpoch)
      },
      utils: Utils,
      types: { Public },
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
    ).to.not.eventually.rejected
  })

  it('should throw when signer is not sender', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: TARGET,
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
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({
      winProb: Utils.computeWinningProbability(0.5)
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
      ticketEpoch: new UINT256(new BN(2))
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
      channelIteration: new UINT256(new BN(2))
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
      balance_a: new Balance(new BN(100)),
      balance_b: new Balance(new BN(0))
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
        amount: new Balance(new BN(100))
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
