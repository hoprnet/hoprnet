import BigNumber from 'bignumber.js'
import { keccak256, MAX_UINT256, createChallenge, signMessage, getChannelId } from './random'

BigNumber.config({ EXPONENTIAL_AT: 1e9 })

/*
  prepares ticket payload
*/
const Ticket = ({
  web3,
  sender,
  receiver,
  porSecret,
  signerPrivKey,
  amount,
  counter,
  winProbPercent,
  counterPartySecret,
}: {
  web3: any
  sender: string
  receiver: string
  porSecret: string // needs to be bytes32
  signerPrivKey: string
  amount: string
  counter: number
  winProbPercent: string // max 100
  counterPartySecret?: string // needs to be bytes32
}): {
  channelId: string // return channel ID
  challenge: string // return hashed alternative
  winProb: string // return winProb in bytes32
  encodedTicket: string // return hashed alternative
  signature: string // signature of encodedTicket
  porSecret: string // same as input
  amount: string // same as input
  hashedCounterPartySecret?: string // return hashed alternative
  counterPartySecret?: string // same as input
  r: string
  s: string
  v: string
} => {
  // proof of relay related hashes
  const challenge = createChallenge(porSecret)

  // proof of randomness related hashes
  let hashedCounterPartySecret: string
  if (counterPartySecret != null) {
    hashedCounterPartySecret = keccak256({
      type: 'bytes27',
      value: counterPartySecret,
    }).slice(0, 56)
  }

  // calculate win probability in bytes32
  const winProb = web3.utils.numberToHex(
    new BigNumber(winProbPercent).multipliedBy(MAX_UINT256).dividedBy(100).toString()
  )

  const channelId = getChannelId(sender, receiver)

  const raw: [string, string, number][] = [
    [receiver, 'bytes', 20],
    [challenge, 'bytes', 32],
    [counter, 'number', 3],
    [amount, 'number', 12],
    [winProb, 'bytes', 32],
  ]

  const encodedTicket = raw
    .reduce((acc, entry) => {
      switch (entry[1]) {
        case 'bytes':
          return acc + (entry[0] as string).replace(/0x/, '').padStart(entry[2] * 2, '0')
        case 'number':
          return acc + new BigNumber(entry[0]).toString(16).padStart(entry[2] * 2, '0')
      }
    }, '0x')
    .toLowerCase()

  const { signature, r, s, v } = signMessage(web3, encodedTicket, signerPrivKey)

  return {
    channelId,
    challenge,
    winProb,
    encodedTicket,
    signature,
    porSecret,
    amount,
    counterPartySecret,
    hashedCounterPartySecret,
    r,
    s,
    v,
  }
}

export { Ticket }
