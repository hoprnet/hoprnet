import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import { stringToU8a } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import PeerId from 'peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import sinon from 'sinon'
import { validateUnacknowledgedTicket } from './tickets'

chai.use(chaiAsPromised)

// target is party A, sender is party B
const TARGET = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
const TARGET_ADDRESS = stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0')
const SENDER = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const SENDER_ADDRESS = stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356')

const createMockTicket = ({
  target = TARGET,
  amount = 1,
  winProb = 1,
  epoch = 1
}: {
  target?: PeerId
  amount?: number
  winProb?: number
  epoch?: number
}) => {
  return ({
    counterparty: target.pubKey.marshal(),
    challenge: new Uint8Array(),
    amount: new BN(amount),
    winProb: new Uint8Array(winProb),
    epoch: new BN(epoch)
  } as unknown) as Types.Ticket
}

const createMockSignedTicket = ({
  sender = SENDER,
  target = TARGET,
  amount = 1,
  winProb = 1
}: {
  sender?: PeerId
  target?: PeerId
  amount?: number
  winProb?: number
}) => {
  return ({
    ticket: createMockTicket({ target, amount, winProb }),
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
  channelBalance = {
    balance_a: new BN(0),
    balance_b: new BN(100)
  },
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
  channelBalance?: {
    balance_a: BN
    balance_b: BN
  }
  getWinProbabilityAsFloat?: number
}) => {
  const pubKeyToAccountId = sinon.stub()
  pubKeyToAccountId.withArgs(sender.pubKey.marshal()).returns(Promise.resolve(senderAddress))
  pubKeyToAccountId.withArgs(target.pubKey.marshal()).returns(Promise.resolve(targetAddress))

  const isPartyA = sinon.stub()
  isPartyA.withArgs(targetAddress, senderAddress).returns(true)
  isPartyA.withArgs(senderAddress, targetAddress).returns(false)

  return ({
    ticketAmount: ticketAmount,
    ticketWinProb: ticketWinProb,
    paymentChannels: {
      account: {
        address: targetAddress,
        ticketEpoch: Promise.resolve(ticketEpoch)
      },
      utils: {
        isPartyA: isPartyA,
        pubKeyToAccountId,
        getWinProbabilityAsFloat: sinon.stub().returns(getWinProbabilityAsFloat)
      },
      channel: {
        isOpen: sinon.stub().returns(Promise.resolve(isChannelOpen)),
        create: isChannelStored
          ? sinon.stub().returns({
              balance_a: Promise.resolve(channelBalance.balance_a),
              balance_b: Promise.resolve(channelBalance.balance_b)
            })
          : sinon.stub().throws()
      }
    }
  } as unknown) as Hopr<Chain>
}

const getAllTicketsMock = async () => []

describe('unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
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
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
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
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
      })
    ).to.eventually.rejectedWith('Ticket amount')
  })

  it('should throw when ticket chance is low', async function () {
    const node = createMockNode({
      getWinProbabilityAsFloat: 0.5
    })
    const signedTicket = createMockSignedTicket({
      winProb: 0.5
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
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
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
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
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
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
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
      })
    ).to.eventually.rejectedWith('does not match our account counter')
  })

  it('should throw if channel does not have enough funds', async function () {
    const node = createMockNode({
      channelBalance: {
        balance_a: new BN(100),
        balance_b: new BN(0)
      }
    })
    const signedTicket = createMockSignedTicket({})

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: getAllTicketsMock
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })

  it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
    const node = createMockNode({})
    const signedTicket = createMockSignedTicket({})
    const ticketsInDb = [
      createMockSignedTicket({
        amount: 100
      })
    ]

    return expect(
      validateUnacknowledgedTicket({
        node,
        senderPeerId: SENDER,
        targetPeerId: TARGET,
        signedTicket,
        getAllTickets: async () => ticketsInDb
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds when you include unredeemed tickets')
  })
})
