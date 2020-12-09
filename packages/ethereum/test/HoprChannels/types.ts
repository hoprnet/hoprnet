export type Account = {
  privKey: string
  pubKey: string
  pubKeyFirstHalf: BN
  pubKeySecondHalf: BN
  address: string
}

export type Ticket = {
  recipient: string
  proofOfRelaySecret: string
  counter: string
  amount: string
  winProb: string
  iteration: string
}
