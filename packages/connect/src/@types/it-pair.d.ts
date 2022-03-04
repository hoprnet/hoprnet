declare module 'it-pair' {
  type SourceType<T> = AsyncIterable<T>

  type Stream<T> = {
    sink: (source: SourceType<T>) => Promise<void>
    source: SourceType<T>
  }

  export default function Pair<T>(): Stream<T>
}
