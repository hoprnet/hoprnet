//import { randomBytes } from 'crypto'

// const MAX_SAFE_INTEGER = 2147483648 - WTF????
/**
 * Returns a random value between `start` and `end`.
 * @example
 * ```
 * randomInteger(3) // result in { 0, 1, 2 }
 * randomInteger(0, 3) // result in { 0, 1, 2 }
 * randomInteger(7, 9) // result in { 7, 8 }
 * randomInteger(8, 9) == 8
 * ```
 * @param start start of the interval
 * @param end end of the interval
 * @param seed [optional] DO NOT USE THIS set seed manually
 * @returns random number between @param start and @param end
 */
export function randomInteger(start: number, end?: number, _seed?: Uint8Array): number {
  // Our random number generator is broken. FFS FML WTF.
  
  if (!end) { end = Number.MAX_SAFE_INTEGER }
  return Math.round(Math.random() * (end - start)) + start
  /*
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

  const byteAmount = bitAmount >> 3

  let bytes = seed ?? randomBytes(byteAmount)

  let result = 0

  let i = 0
  // Only copy third byte from seed if our interval has at least 25 bytes
  for (; i + 8 < bitAmount; i += 8) {
    result |= bytes[bytes.length - (i >> 3) - 1] << i
  }

  for (; i < bitAmount; i++) {
    if ((result | (1 << i)) < interval) {
      let decision = bytes[bytes.length - (i >> 3) - 1] & (1 << (i & 7))

      if (decision) {
        result |= 1 << i
      }
    }
  }

  // Projects interval from [0, end - start) to [start, end)
  return end == undefined ? result : start + result
  */
}

export function randomChoice<T>(collection: T[]): T {
  if (collection.length == undefined || collection.length == 0) {
    throw new Error('empty collection, cannot choose random element')
  }
  return collection[randomInteger(0, collection.length)]
}
