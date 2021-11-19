declare module 'it-pair' {
  type Stream<T> = {
    sink: (source: Stream<T>['source']) => Promise<void>
    source: AsyncGenerator<T, void>
  }

  export default function Pair<T>(): Stream<T>
}
