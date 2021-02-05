import BN from 'bn.js'
import { ChannelEntry, AccountEntry } from '../types'
import { BYTES27_LENGTH } from '../constants'

export const CHANNEL_ENTRY = new ChannelEntry(undefined, {
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  deposit: new BN(4),
  partyABalance: new BN(5),
  closureTime: new BN(6),
  stateCounter: new BN(7),
  closureByPartyA: true
})

export const ACCOUNT_ENTRY = new AccountEntry(undefined, {
  blockNumber: new BN(1),
  transactionIndex: new BN(2),
  logIndex: new BN(3),
  hashedSecret: new Uint8Array(Buffer.from([1, 2, 4, 5]), undefined, BYTES27_LENGTH),
  counter: new BN(4)
})
