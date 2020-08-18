declare namespace Hash {
  const SIZE: number
}
declare interface Hash extends Uint8Array {
  new (hash: Uint8Array, ...props: any[]): Hash
}

export default Hash
