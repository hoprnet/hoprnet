declare module 'stream-to-it' {
  type Readable = import('stream').Readable
  type Writable = import('stream').Writable
  type Duplex = import('stream').Duplex

  type SourceType<T> = AsyncIterable<T> | Iterable<T>

  export function sink<T>(stream: Writable): (source: SourceType<T>) => Promise<void>
  export function source<T>(stream: Readable): SourceType<T>
  export function duplex<T>(stream: Duplex): {
    sink: (source: SourceType<T>) => Promise<void>
    source: SourceType<T>
  }
}
