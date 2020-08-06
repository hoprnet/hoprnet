declare namespace Signature {
  const SIZE: number

  function create(
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

export default Signature
