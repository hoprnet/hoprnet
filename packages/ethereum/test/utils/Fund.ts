import { stringToU8a } from '@hoprnet/hopr-utils'
import { encode, signMessage } from './random'

type IFund = (args: {
  stateCounter: string
  initiator: string
  deposit: string
  partyAAmount: string
  notAfter: string
  signerPrivKey: string
}) => {
  encodedFund: string
  signature: Uint8Array // signature of hashedTicket
  r: Uint8Array
  s: Uint8Array
  v: number
}

/*
  prepares fund payload
*/
const Fund: IFund = ({ stateCounter, initiator, deposit, partyAAmount, notAfter, signerPrivKey }) => {
  const encodedFund = encode([
    { type: 'uint256', value: stateCounter },
    { type: 'address', value: initiator },
    { type: 'uint256', value: deposit },
    { type: 'uint256', value: partyAAmount },
    { type: 'uint256', value: notAfter }
  ])

  const { signature, r, s, v } = signMessage(encodedFund, stringToU8a(signerPrivKey))

  return {
    encodedFund,
    signature,
    r,
    s,
    v
  }
}

export { Fund }
