import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a, randomInteger, u8aToHex } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { Ticket, Hash, Balance, SignedTicket, UINT256 } from '.'
import { pubKeyToAddress, privKeyToPubKey } from '../utils'
import * as testconfigs from '../config.spec'

const WIN_PROB = new BN(1)

describe('test signedTicket construction', async function () {
  const [, userB] = await Promise.all(
    testconfigs.DEMO_ACCOUNTS.slice(0, 2).map(
      async (str: string) => await pubKeyToAddress(await privKeyToPubKey(stringToU8a(str)))
    )
  )

  const [userAPrivKey] = testconfigs.DEMO_ACCOUNTS.slice(0, 2).map((str: string) => stringToU8a(str))

  const userAPubKey = await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))

  it('should create new signedTicket using struct', async function () {
    const challenge = new Hash(randomBytes(32))
    const epoch = UINT256.fromString('0')
    const amount = new Balance(new BN(15))
    const winProb = new Hash(
      new Uint8Array(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
    )
    const channelIteration = UINT256.fromString('0')

    const ticket = new Ticket(userB, challenge, epoch, amount, winProb, channelIteration)
    const signature = await ticket.sign(userAPrivKey)

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket
    })

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(new Hash(userAPubKey)), 'signer incorrect')

    let exponent = randomInteger(0, 7)
    let index = randomInteger(0, signedTicket.length - 1)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verify(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicket)}>, byte #${index}, bit #${exponent}`)
    }
  })
})
