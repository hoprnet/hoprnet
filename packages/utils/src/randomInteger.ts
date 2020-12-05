import { randomBytes } from 'crypto'

const MAX_SAFE_INTEGER = 2147483648
/**
 * @param start
 * @param end
 * @returns random number between @param start and @param end
 */
export function randomInteger(start: number, end?: number, seed?: Uint8Array): number {
  if (start < 0 || (end != undefined && end < 0)) {
    throw Error(`'start' and 'end' must be positive.`)
  }

  if (end != undefined) {
    if (start >= end) {
      throw Error(`Invalid interval. 'end' must be strictly greater than 'start'. Got start: <${start}> end: <${end}>`)
    }

    if (start + 1 == end) {
      return start
    }
  } else {
    if (start == 0) {
      throw Error(`Cannot pick a random number that is >= 0 and < 0`)
    }
  }

  // Projects interval from [start, end) to [0, end - start)
  let interval = end == undefined ? start : end - start

  if (interval == 1) {
    return start
  }

  if (interval > MAX_SAFE_INTEGER) {
    throw Error(`Not implemented`)
  }

  const bitAmount = 32 - Math.clz32(interval - 1)

  const byteAmount = Math.ceil(bitAmount / 8)

  let bytes = seed ?? randomBytes(byteAmount)

  let result = 0

  for (let i = 0; i < bitAmount; i++) {
    if ((result | (1 << i)) < interval) {
      let index = Math.floor(i / 8)
      let offset = i % 8
      let decision = bytes[bytes.length - index - 1] & (1 << offset)

      if (decision) {
        result |= 1 << i
      }
    }
  }

  // Projects interval from [0, end - start) to [start, end)
  return end == null ? result : start + result
}

export function randomChoice<T>(collection: T[]): T {
  if (collection.length == undefined || collection.length == 0) {
    throw new Error('empty collection, cannot choose random element')
  }
  return collection[randomInteger(0, collection.length)]
}
