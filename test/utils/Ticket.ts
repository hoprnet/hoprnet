import BigNumber from 'bignumber.js'
import { keccak256, MAX_UINT256, encode, createChallenge, signMessage, getChannelId, getParties } from './random'

BigNumber.config({ EXPONENTIAL_AT: 1e9 })

type ITicket = (args: {
  web3: any
  accountA: string
  accountB: string
  porSecret: string // needs to be bytes32
  signerPrivKey: string
  counterPartySecret: string // needs to be bytes32
  amount: string
  counter: number
  winProbPercent: string // max 100
}) => {
  accountA: string // return same as provided
  accountB: string // return same as provided
  porSecret: string // return same as provided
  counterPartySecret: string // return same as provided
  amount: string // return same as provided
  counter: number // return same as provided
  channelId: string // return channel ID
  challenge: string // return hashed alternative
  hashedCounterPartySecret: string // return hashed alternative
  winProb: string // return winProb in bytes32
  encodedTicket: string // return hashed alternative
  signature: string // signature of encodedTicket
  r: string
  s: string
  v: string
}

/*
  prepares ticket payload
*/
const Ticket: ITicket = ({
  web3,
  accountA,
  accountB,
  porSecret,
  signerPrivKey,
  counterPartySecret,
  amount,
  counter,
  winProbPercent,
}) => {
  // proof of relay related hashes
  const challenge = createChallenge(porSecret)

  // proof of randomness related hashes
  const hashedCounterPartySecret = keccak256({
    type: 'bytes27',
    value: counterPartySecret,
  }).slice(0, 56)

  // calculate win probability in bytes32
  const winProb = web3.utils.numberToHex(
    new BigNumber(winProbPercent).multipliedBy(MAX_UINT256).dividedBy(100).toString()
  )

  const { partyA, partyB } = getParties(accountA, accountB)
  const channelId = getChannelId(partyA, partyB)

  const encodedTicket = encode([
    { type: 'bytes32', value: channelId },
    { type: 'bytes32', value: challenge },
    { type: 'bytes27', value: hashedCounterPartySecret },
    { type: 'uint32', value: counter },
    { type: 'uint256', value: amount },
    { type: 'bytes32', value: winProb },
  ])

  const { signature, r, s, v } = signMessage(web3, encodedTicket, signerPrivKey)

  return {
    accountA,
    accountB,
    porSecret,
    counterPartySecret,
    amount,
    counter,
    channelId,
    challenge,
    hashedCounterPartySecret,
    winProb,
    encodedTicket,
    signature,
    r,
    s,
    v,
  }
}

export { Ticket }
