declare module 'it-handshake' {
  type Stream = import('libp2p').Stream

  export type Handshake<T> = {
    reader: {
      next(bytes: number): Promise<T>
    }
    writer: {
      end(): void
      push(msg: T): void
    }
    stream: Stream
    rest(): void
    write(msg: T): void
    read(): Promise<T>
  }
  export default function <T>(stream: Stream): Handshake<T>
}
