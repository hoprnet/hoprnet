declare module 'it-handshake' {
  type Stream<T> = {
    sink: (source: Stream<T>['source']) => Promise<void>
    source: AsyncGenerator<T, void>
  }

  export type Handshake<T> = {
    reader: {
      next(bytes: number): Promise<T>
    }
    writer: {
      end(): void
      push(msg: T): void
    }
    stream: Stream<T>
    rest(): void
    write(msg: T): void
    read(): Promise<T>
  }
  export default function <T>(stream: Stream<T>): Handshake<T>
}
