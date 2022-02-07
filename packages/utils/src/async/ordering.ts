import { FIFO } from '../collection'

type Item<T> = {
  index: number
  value?: T
}

/**
 * Creates a queue that consumes items asynchronously and potentially
 * unorders but outputs them ordered using an asynchronous iterator.
 * Each element consists of a value and an index upon which
 * elements are ordered.
 * @example
 * import { ordered, wait } from '@hoprnet/hopr-utils'
 *
 * const order = ordered<number>()
 *
 * (async function () {
 *   order.push({ index: 0, value: 'first' })
 *   wait(50)
 *   order.push({ index: 2, value: 'second' })
 *   wait(50)
 *   order.push({ index: 1, value: 'third' })
 *   wait(50)
 *   order.end()
 * })()
 *
 * const result: string[] = []
 * for await (const item of order.iterator()) {
 *   result.push(item.value)
 * }
 * // result == ['first', 'third', 'second']
 * @returns an ordered stream
 */
export function ordered<T>() {
  const queue = FIFO<Item<T>>()

  let resolve: (done: boolean) => void

  let next = new Promise<boolean>((_resolve) => (resolve = _resolve))

  let currentIndex: number | undefined

  function push(newItem: Item<T>) {
    if ([undefined, newItem.index].includes(currentIndex)) {
      queue.push(newItem)
      currentIndex = newItem.index + 1
      resolve(false)
    } else if (newItem.index < currentIndex) {
      queue.replace(
        (item: Item<T>) => {
          return item.index == newItem.index
        },
        () => newItem
      )
    } else {
      while (currentIndex < newItem.index) {
        queue.push({
          index: currentIndex
        })
        currentIndex++
      }
      queue.push(newItem)
      currentIndex++
    }

    // console.log(queue.toArray())
    if (queue.peek().value != undefined) {
      resolve(false)
    }
  }

  function end() {
    resolve(true)
  }

  async function* iterator() {
    while (true) {
      if (await next) {
        break
      }
      next = new Promise<boolean>((_resolve) => (resolve = _resolve))
      while (queue.peek()?.value != undefined) {
        yield queue.shift()
      }
    }
  }

  return {
    push,
    iterator,
    end
  }
}
