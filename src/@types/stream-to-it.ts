declare module 'stream-to-it' {
  type Readable = import('stream').Readable
  type Writable = import('stream').Writable
  type Duplex = import('stream').Duplex

  export function sink<T>(stream: Writable): (source: AsyncGenerator<T, T | void>) => Promise<void>
  export function source<T>(stream: Readable): AsyncGenerator<T, T | void>
  export function duplex<T>(
    stream: Duplex
  ): {
    sink: (stream: AsyncGenerator<T, void>) => Promise<void>
    source: AsyncGenerator<T, void>
  }
}
