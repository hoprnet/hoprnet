import type Chain from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import BN from 'bn.js'
import PeerId from 'peer-id'
import chaiAsPromised from 'chai-as-promised'
import chai, { expect } from 'chai'
import sinon from 'sinon'
import { validateUnacknowledgedTicket } from './tickets'

chai.use(chaiAsPromised)

const createTicket = ({ sender, amount = 1, winProb = 1 }: { sender: PeerId; amount: number; winProb?: number }) => {
  return ({
    counterparty: sender.pubKey.marshal(),
    challenge: sender.pubKey.marshal(),
    epoch: new BN(1),
    amount: new BN(amount),
    winProb: new Uint8Array(winProb)
  } as unknown) as Types.Ticket
}

const createSignedTicket = ({
  sender,
  amount = 1,
  winProb = 1
}: {
  sender: PeerId
  amount?: number
  winProb?: number
}) => {
  return ({
    ticket: createTicket({ sender, amount, winProb }),
    signer: Promise.resolve(sender.pubKey.marshal())
  } as unknown) as Types.SignedTicket
}

const createNode = ({
  sender,
  target,
  ticketAmount = 1,
  ticketWinProb = 1,
  isChannelOpen = true,
  isChannelStored = true,
  getWinProbabilityAsFloat = 1
}: {
  sender: PeerId
  target: PeerId
  ticketAmount?: number
  ticketWinProb?: number
  isChannelOpen?: boolean
  isChannelStored?: boolean
  getWinProbabilityAsFloat?: number
}) => {
  return ({
    ticketAmount: ticketAmount,
    ticketWinProb: ticketWinProb,
    paymentChannels: {
      account: {
        address: target.pubKey.marshal()
      },
      utils: {
        isPartyA: sinon
          .stub()
          .withArgs(target.pubKey.marshal(), sender.pubKey.marshal())
          .returns(Promise.resolve(true)),
        pubKeyToAccountId: sinon.stub().withArgs(sender.pubKey.marshal()).returns(Promise.resolve('he')),
        getWinProbabilityAsFloat: sinon.stub().returns(getWinProbabilityAsFloat)
      },
      channel: {
        isOpen: sinon.stub().returns(Promise.resolve(isChannelOpen)),
        create: isChannelStored
          ? sinon.stub().returns({
              balance_a: 100,
              balance_b: 50
            })
          : sinon.stub().throws()
      }
    }
  } as unknown) as Hopr<Chain>
}

describe('unit test validateUnacknowledgedTicket', function () {
  const sender = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
  const target = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')

  it('should throw when signer is not sender', async function () {
    const node = createNode({
      sender,
      target
    })
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        signedTicket,
        senderPeerId: await PeerId.create()
      })
    ).to.eventually.rejectedWith('The signer of the ticket does not match the sender')
  })

  it('should throw when ticket amount is low', async function () {
    const node = createNode({
      sender,
      target,
      ticketAmount: 2
    })
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        signedTicket,
        senderPeerId: sender
      })
    ).to.eventually.rejectedWith('is lower than')
  })

  it('should throw when ticket chance is low', async function () {
    const node = createNode({
      sender,
      target,
      getWinProbabilityAsFloat: 0.5
    })
    const signedTicket = createSignedTicket({
      sender,
      winProb: 0.5
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        signedTicket,
        senderPeerId: sender
      })
    ).to.eventually.rejectedWith('is lower than')
  })

  it('should throw if there no channel open', async function () {
    const node = createNode({
      sender,
      target,
      isChannelOpen: false
    })
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        signedTicket,
        senderPeerId: sender
      })
    ).to.eventually.rejectedWith('is not open')
  })

  it('should throw if channel is not stored', async function () {
    const node = createNode({
      sender,
      target,
      isChannelStored: false
    })
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateUnacknowledgedTicket({
        node,
        signedTicket,
        senderPeerId: sender
      })
    ).to.eventually.rejectedWith('not found')
  })
})
