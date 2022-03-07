declare module 'it-handshake' {
  type SourceType<T> = AsyncIterable<T> | Iterable<T>

  type Stream<T> = {
    sink: (source: SourceType<T>) => Promise<void>
    source: SourceType<T>
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
