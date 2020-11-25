import { randomBytes } from 'crypto'

const MAX_SAFE_INTEGER = 2147483647
/**
 * @param start
 * @param end
 * @returns random number between @param start and @param end
 */
export function randomInteger(start: number, end?: number): number {
  if (start < 0 || (end != null && end < 0)) {
    throw Error(`'start' and 'end' must be positive.`)
  }

  if (end != null) {
    if (start >= end) {
      throw Error(`Invalid interval. 'end' must be strictly greater than 'start'.`)
    }

    if (start + 1 == end) {
      return start
    }
  }

  // Projects interval from [start, end) to [0, end - start)
  let interval = end == null ? start : end - start

  if (interval > MAX_SAFE_INTEGER) {
    throw Error(`Not implemented`)
  }

  const bitAmount = 32 - Math.clz32(interval - 1)

  const byteAmount = Math.ceil(bitAmount / 8)

  let bytes = randomBytes(byteAmount)

  let bitCounter = 0
  let byteCounter = 0

  const nextBit = (): number => {
    let result = bytes[byteCounter] % 2
    bytes[byteCounter] = bytes[byteCounter] >> 1
    if (++bitCounter == 8) {
      bitCounter = 0
      byteCounter++
    }
    return result
  }

  let result = 0
  for (let i = 0; i < byteAmount; i++) {
    for (let j = 0; j < 8; j++) {
      if ((result | (1 << (i * 8 + j))) < interval) {
        if (nextBit()) {
          result |= 1 << (i * 8 + j)
        }
      }
    }
  }

  return end == null ? result : start + result
}

export function randomChoice<T>(collection: T[]): T {
  if (collection.length === 0) {
    throw new Error('empty collection, cannot choose random element')
  }
  return collection[randomInteger(0, collection.length)]
}
