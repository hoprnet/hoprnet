type Entry<T> = {
  data: T
  next: Entry<T> | undefined
}

/**
 *
 * @returns
 */
export function FIFO<T>() {
  let head: Entry<T> | undefined
  let tail: Entry<T> | undefined
  let length = 0

  function size() {
    return length
  }

  function shift(): T | undefined {
    if (length == 0) {
      return undefined
    }
    const result = head.data

    head = head.next

    length--

    return result
  }

  function push(data: T): number {
    if (length == 0) {
      head = {
        data,
        next: undefined
      }
      tail = head
    } else {
      tail.next = {
        data,
        next: undefined
      }
      tail = tail.next
    }

    return ++length
  }

  return { push, shift, size }
}
