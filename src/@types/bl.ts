declare module 'bl' {
  interface BL extends Uint8Array {}

  interface BLConstructor {
    new (arg: BL | Uint8Array | (BL | Uint8Array)[]): BL
  }

  var BL: BLConstructor

  export default BL
}
