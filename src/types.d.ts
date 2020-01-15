import BN from 'bn.js'

declare class SignedTicket extends Uint8Array {
  ticket: Types['Ticket']
  signature: Uint8Array
}

export default interface Types {
  Balance: BN
  Hash: Uint8Array
  Moment: BN
  Ticket: Uint8Array
  AccountId: Uint8Array
  State: any
  SignedTicket: SignedTicket
}
