declare module 'it-handshake' {
  type Stream = import('libp2p').Stream

  export type Handshake = {
    reader: {
      next(bytes: number): Promise<Uint8Array>
    }
    writer: {
      end(): void
      push(msg: Uint8Array)
    }
    stream: Stream
    rest(): void
    write(msg: Uint8Array): void
    read(): Promise<Uint8Array>
  }
  export default function (stream: Stream): Handshake
}
