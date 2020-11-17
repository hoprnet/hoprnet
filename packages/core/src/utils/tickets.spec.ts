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

chai.use(chaiAsPromised)

// target is party A, sender is party B
const target = PeerId.createFromB58String('16Uiu2HAm5g4fTADcjPQrtp9LtN2wCmPJTQPD7vMnWCZp4kwKCVUT')
const sender = PeerId.createFromB58String('16Uiu2HAmM9KAPaXA4eAz58Q7Eb3LEkDvLarU4utkyLwDeEK6vM5m')
const targetAddress = stringToU8a('0xf3a509473be4bcd8af0d1961d75a5a3dc9e47ba0')
const senderAddress = stringToU8a('0x65e78d07acf7b654e5ae6777a93ebbf30f639356')

const createTicket = ({
  sender,
  amount = 1,
  winProb = 1,
  epoch = 1
}: {
  sender: PeerId
  amount?: number
  winProb?: number
  epoch?: number
}) => {
  return ({
    counterparty: sender.pubKey.marshal(),
    challenge: sender.pubKey.marshal(),
    amount: new BN(amount),
    winProb: new Uint8Array(winProb),
    epoch: new BN(epoch)
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
  sender: PeerId
  target: PeerId
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

describe('unit test validateUnacknowledgedTicket', function () {
  it('should pass if ticket is okay', async function () {
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
        senderPeerId: sender
      })
    ).to.eventually.to.not.rejected
  })

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

  it('should throw if ticket epoch does not match our account counter', async function () {
    const node = createNode({
      sender,
      target,
      ticketEpoch: new BN(2)
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
    ).to.eventually.rejectedWith('does not match our account counter')
  })

  it('should throw if channel does not have enough funds', async function () {
    const node = createNode({
      sender,
      target,
      channelBalance: {
        balance_a: new BN(100),
        balance_b: new BN(0)
      }
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
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })

  // @TODO: implement test
  // it('should throw if channel does not have enough funds when you include unredeemed tickets', async function () {
  //   const node = createNode({
  //     sender,
  //     target
  //   })
  //   const signedTicket = createSignedTicket({
  //     sender
  //   })

  //   return expect(
  //     validateUnacknowledgedTicket({
  //       node,
  //       signedTicket,
  //       senderPeerId: sender
  //     })
  //   ).to.eventually.rejectedWith('Payment channel does not have enough funds when you include unredeemed tickets')
  // })
})

describe('unit test validateCreatedTicket', function () {
  it('should pass if ticket is okay', async function () {
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateCreatedTicket({
        myBalance: new BN(1),
        signedTicket
      })
    ).to.eventually.to.not.rejected
  })

  it('should throw when signer is not sender', async function () {
    const signedTicket = createSignedTicket({
      sender
    })

    return expect(
      validateCreatedTicket({
        myBalance: new BN(0),
        signedTicket
      })
    ).to.eventually.rejectedWith('Payment channel does not have enough funds')
  })
})
