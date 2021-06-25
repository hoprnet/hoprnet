export function eagerIterator<T>(iterator: AsyncIterator<T>): AsyncGenerator<T> {
  let result = iterator.next()
  let received: IteratorResult<T>

  return (async function* () {
    while (true) {
      received = await result

      if (received.done) {
        break
      }
      result = iterator.next()
      yield received.value
    }
  })()
}
