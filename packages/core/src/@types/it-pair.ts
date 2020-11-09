declare module 'it-pair' {
  type Stream = import('libp2p').Stream

  export default function Pair(): Stream
}
