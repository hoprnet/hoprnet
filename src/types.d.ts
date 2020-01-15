import BN from 'bn.js'

export default interface Types {
  Balance: BN
  Hash: Uint8Array
  Moment: BN
  Ticket: Uint8Array
  AccountId: Uint8Array
  State: any
  SignedTicket: {
    ticket: Types['Ticket']
    signature: Uint8Array
  }
}
