declare interface HashStatic {
  readonly SIZE: number
  new (hash: Uint8Array, ...props: any[]): Hash
}
declare interface Hash extends Uint8Array {}

declare var Hash: HashStatic

export default Hash
