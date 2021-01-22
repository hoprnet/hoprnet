declare module 'bl' {
  interface BLInterface {
    slice(): Uint8Array
  }

  type BLArg = BLInterface | Uint8Array | Buffer
  interface BLConstructor {
    new (arg: BLArg | BLArg[]): BLInterface
  }

  var BL: BLConstructor

  export type { BLInterface }
  export default BL
}
