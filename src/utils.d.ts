import { AccountId, Hash, Signature } from './types'

declare interface Utils {
  /**
   * Decides whether we take the role of partyA in the channel with `counterparty`.
   * @param self id of ourself
   * @param counterparty id of the counterparty
   */
  isPartyA(self: AccountId.Instance, counterparty: AccountId.Instance): boolean

  /**
   * Returns the Id of the channel between ourself and `counterparty`.
   * @param self id of ourself
   * @param counterparty id of the counterparty
   * @param props additional arguments
   */
  getId(self: AccountId.Instance, counterparty: AccountId.Instance, ...props: any[]): Promise<Hash.Instance>

  /**
   * Converts a public key into an on-chain AccountId (e.g. an Ethereum address).
   * @param pubkey a public key
   * @param args additional arguments
   */
  pubKeyToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<AccountId.Instance>

  /**
   * Uses the native on-chain hash function to compute a hash value of `msg`.
   * @param msg message to hash
   */
  hash(msg: Uint8Array): Promise<Hash.Instance>

  /**
   * Uses the native on-chain signature scheme to create an on-chain verifiable signature.
   * @param msg message to sign
   * @param privKey private key of the signer
   * @param pubKey public key of the signer
   */
  sign(
    msg: Uint8Array,
    privKey: Uint8Array,
    pubKey: Uint8Array
  ): Promise<Signature.Instance>

  /**
   * Uses the native on-chain signature scheme to check a signature for its validity.
   * @param msg message to verify
   * @param signature signature over `msg` to verify
   * @param pubkey public key of the signer
   */
  verify(msg: Uint8Array, signature: Signature.Instance, pubkey: Uint8Array): Promise<boolean>
}

export default Utils