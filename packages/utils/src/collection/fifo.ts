type Entry<T> = {
  data: T
  next: Entry<T> | undefined
}

export interface FIFO<T> {
  last(): T
  size(): number
  shift(): T | undefined
  peek(): T | undefined
  replace(find: (item: T) => boolean, modify: (oldItem: T) => T): boolean
  push(item: T): number
  toArray(): T[]
}
/**
 *
 * @returns
 */
export function FIFO<T>(): FIFO<T> {
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

    if (length == 0) {
      tail = undefined
    }

    return result
  }

  function peek(): T | undefined {
    return head?.data
  }

  function last(): T | undefined {
    return tail?.data
  }

  function replace(find: (item: T) => boolean, modify: (oldItem: T) => T): boolean {
    let current: Entry<T> | undefined = head

    let found = false
    while (current != undefined) {
      if (find(current.data)) {
        current.data = modify(current.data)
        found = true
      }

      if (found) {
        break
      } else {
        current = current.next
      }
    }

    return found
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

  function toArray() {
    const result: T[] = []

    let current = head

    while (current != undefined) {
      result.push(current.data)
      current = current.next
    }

    return result
  }

  return { last, peek, push, replace, shift, size, toArray }
}
