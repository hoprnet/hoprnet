import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a, randomInteger, u8aToHex } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { AccountId, Ticket, Hash, TicketEpoch, Balance, SignedTicket } from '.'
import { pubKeyToAccountId, privKeyToPubKey } from '../utils'
import * as testconfigs from '../config.spec'

const WIN_PROB = new BN(1)

const generateTicketData = async (receiver: AccountId) => {
  const challenge = new Hash(randomBytes(32))
  const epoch = new TicketEpoch(0)
  const amount = new Balance(15)
  const winProb = new Hash(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
  const onChainSecret = new Hash(randomBytes(27))
  const channelIteration = new TicketEpoch(0)

  return {
    counterparty: receiver,
    challenge,
    epoch,
    amount,
    winProb,
    onChainSecret,
    channelIteration
  }
}

describe('test signedTicket construction', async function () {
  const [, userB] = await Promise.all(
    testconfigs.DEMO_ACCOUNTS.slice(0, 2).map(
      async (str: string) => await pubKeyToAccountId(await privKeyToPubKey(stringToU8a(str)))
    )
  )

  const [userAPrivKey] = testconfigs.DEMO_ACCOUNTS.slice(0, 2).map((str: string) => stringToU8a(str))

  const userAPubKey = await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))

  it('should create new signedTicket using struct', async function () {
    const { counterparty, challenge, epoch, amount, winProb, channelIteration } = await generateTicketData(userB)
    const ticket = new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration)
    const signedTicket = await ticket.sign(userAPrivKey) 

    assert(await signedTicket.verifySignature(userAPubKey))
    assert(new Hash(await signedTicket.getSigner()).eq(userAPubKey), 'signer incorrect')
    assert(signedTicket.ticket.counterparty.eq(userB), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(winProb), 'wrong winProb')

    let exponent = randomInteger(0, 7)
    let index = randomInteger(0, signedTicket.serialize().length - 1)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verifySignature(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicket.serialize())}>, byte #${index}, bit #${exponent}`)
    }
  })


  it('serialize / deserialize', async function () {
    const { counterparty, challenge, epoch, amount, winProb, channelIteration }= await generateTicketData(userB)
    const temp = await (new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration).sign(userAPrivKey))
    const serialized = temp.serialize()
    const signedTicket = SignedTicket.deserialize(serialized)
    assert(await signedTicket.verifySignature(userAPubKey))
    assert(new Hash(await signedTicket.getSigner()).eq(userAPubKey), 'signer incorrect')
    assert(signedTicket.ticket.counterparty.eq(userB), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(winProb), 'wrong winProb')

    let exponent = randomInteger(0, 7)
    let index = randomInteger(0, signedTicket.serialize().length - 1)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verifySignature(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicket.serialize())}>, byte #${index}, bit #${exponent}`)
    }
  })
})
