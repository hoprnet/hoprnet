import BigNumber from 'bignumber.js'
import { keccak256, xorBytes32, signMessage, MAX_UINT256 } from './random'

BigNumber.config({ EXPONENTIAL_AT: 1e9 })

type ITicket = (args: {
  web3: any
  accountA: string
  accountB: string
  porSecretA: string // needs to be bytes32
  porSecretB: string // needs to be bytes32
  signerPrivKey: string
  counterPartySecret: string // needs to be bytes32
  amount: string
  counter: number
  winProbPercent: string // max 100
}) => {
  accountA: string // return same as provided
  accountB: string // return same as provided
  porSecretA: string // return same as provided
  porSecretB: string // return same as provided
  counterPartySecret: string // return same as provided
  amount: string // return same as provided
  counter: number // return same as provided
  hashedPorSecretA: string // return hashed alternative
  hashedPorSecretB: string // return hashed alternative
  challenge: string // return hashed alternative
  hashedCounterPartySecret: string // return hashed alternative
  winProb: string // return winProb in bytes32
  hashedTicket: string // return hashed alternative
  signature: string // signature of hashedTicket
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
  porSecretA,
  porSecretB,
  signerPrivKey,
  counterPartySecret,
  amount,
  counter,
  winProbPercent
}) => {
  // proof of relay related hashes
  const hashedPorSecretA = keccak256({ type: 'bytes32', value: porSecretA })
  const hashedPorSecretB = keccak256({ type: 'bytes32', value: porSecretB })
  const challenge = xorBytes32(hashedPorSecretA, hashedPorSecretB)

  // proof of randomness related hashes
  const hashedCounterPartySecret = keccak256({
    type: 'bytes32',
    value: counterPartySecret
  })

  // calculate win probability in bytes32
  const winProb = web3.utils.numberToHex(
    new BigNumber(winProbPercent)
      .multipliedBy(MAX_UINT256)
      .dividedBy(100)
      .toString()
  )

  const hashedTicket = keccak256(
    { type: 'bytes32', value: challenge },
    { type: 'bytes32', value: counterPartySecret },
    { type: 'uint256', value: counter },
    { type: 'uint256', value: amount },
    { type: 'bytes32', value: winProb }
  )

  const { signature, r, s, v } = signMessage(web3, hashedTicket, signerPrivKey)

  return {
    accountA,
    accountB,
    porSecretA,
    porSecretB,
    counterPartySecret,
    amount,
    counter,
    hashedPorSecretA,
    hashedPorSecretB,
    challenge,
    hashedCounterPartySecret,
    winProb,
    hashedTicket,
    signature,
    r,
    s,
    v
  }
}

export { Ticket }
