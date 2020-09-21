declare interface SignatureStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      onChainSignature: Uint8Array
      signature: Uint8Array
      recovery: number
      msgPrefix?: Uint8Array
    }
  ): Promise<Signature>
}
declare interface Signature extends Uint8Array {
  onChainSignature: Uint8Array
  signature: Uint8Array
  recovery: number
  msgPrefix: Uint8Array
}

declare var Signature: SignatureStatic

export default Signature
