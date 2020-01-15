import Types from './types'

export default interface Utils {
  isPartyA(self: Types['AccountId'], counterparty: Types['AccountId']): boolean
  getId(self: Types['AccountId'], counterparty: Types['AccountId'], ...props: any[]): Promise<Types['Hash']>
  pubKeyToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<Types['AccountId']>
  hash(msg: Uint8Array): Promise<Types['Hash']>
}
