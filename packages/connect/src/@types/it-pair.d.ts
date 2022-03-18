declare module 'it-pair' {
  type SourceType<T> = AsyncIterable<T> | Iterable<T>

  type Stream<T> = {
    sink: (source: SourceType<T>) => Promise<void>
    source: SourceType<T>
  }

  export default function Pair<T>(): Stream<T>
}

declare module 'it-pair/duplex' {
  type SourceType<T> = AsyncIterable<T> | Iterable<T>

  type Stream<T> = {
    sink: (source: SourceType<T>) => Promise<void>
    source: SourceType<T>
  }

  export default function DuplexPair<T>(): [Stream<T>, Stream<T>]
}
