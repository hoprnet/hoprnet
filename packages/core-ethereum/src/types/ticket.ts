import type { Ticket as ITicket } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { stringToU8a, u8aToHex, u8aConcat } from '@hoprnet/hopr-utils'
import { AccountId, Balance, Hash, SignedTicket, TicketEpoch } from '.'
import { sign } from '../utils'

import Web3 from 'web3'
const web3 = new Web3()

const EPOCH_SIZE = 3
const AMOUNT_SIZE = 12

/**
 * Given a message, prefix it with "\x19Ethereum Signed Message:\n" and return it's hash
 */
function toEthSignedMessageHash(msg: string): Hash {
  const messageWithHOPR = u8aConcat(stringToU8a(Web3.utils.toHex('HOPRnet')), stringToU8a(msg))
  const messageWithHOPRHex = u8aToHex(messageWithHOPR)
  return new Hash(stringToU8a(web3.eth.accounts.hashMessage(messageWithHOPRHex)))
}

class Ticket implements ITicket {
  constructor(
    public counterparty: AccountId,
    public challenge: Hash,
    public epoch: TicketEpoch,
    public readonly amount: Balance,
    public readonly winProb: Hash,
    public channelIteration: TicketEpoch
  ) {}

  serialize(): Uint8Array {
    const serialized = new Uint8Array(Ticket.SIZE())
    const epochBuffer = new Uint8Array(this.epoch.toBuffer('be', EPOCH_SIZE))
    let i = 0
    serialized.set(this.counterparty, i)
    i += AccountId.SIZE
    serialized.set(this.challenge, i)
    i += Hash.SIZE
    serialized.set(epochBuffer, i)
    i += EPOCH_SIZE
    serialized.set(new Uint8Array(this.amount.toBuffer('be', AMOUNT_SIZE)), i)
    i += AMOUNT_SIZE
    serialized.set(this.winProb, i)
    i += Hash.SIZE
    serialized.set(new Uint8Array(this.channelIteration.toBuffer('be', EPOCH_SIZE)), i)
    return serialized
  }

  static deserialize(arr: Uint8Array): Ticket {
    const buffer = arr.buffer
    let i = arr.byteOffset
    const counterparty = new AccountId(new Uint8Array(buffer, i, AccountId.SIZE))
    i += AccountId.SIZE
    const challenge = new Hash(new Uint8Array(buffer, i, Hash.SIZE))
    i += Hash.SIZE
    const epoch = new TicketEpoch(new Uint8Array(buffer, i, EPOCH_SIZE))
    i += EPOCH_SIZE
    const amount = new Balance(new Uint8Array(buffer, i, AMOUNT_SIZE))
    i += AMOUNT_SIZE
    const winProb = new Hash(new Uint8Array(buffer, i, Hash.SIZE))
    i += Hash.SIZE
    const channelIteration = new TicketEpoch(new Uint8Array(buffer, i, EPOCH_SIZE))
    return new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration)
  }

  get hash(): Promise<Hash> {
    return Promise.resolve(toEthSignedMessageHash(u8aToHex(this.serialize())))
  }

  static SIZE(): number {
    return AccountId.SIZE + Hash.SIZE + EPOCH_SIZE + AMOUNT_SIZE + Hash.SIZE + EPOCH_SIZE
  }

  getEmbeddedFunds(): Balance {
    return new Balance(this.amount.mul(new BN(this.winProb)).div(new BN(new Uint8Array(Hash.SIZE).fill(0xff))))
  }


  async sign(privKey: Uint8Array): Promise<SignedTicket> {
    // TODO
    return new SignedTicket(this, await sign(await this.hash, privKey, null, null))
  }
}

export default Ticket
