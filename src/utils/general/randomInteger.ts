import { randomBytes } from 'crypto'

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

  // Projects interval from [start, end] to [0, end - start]
  let interval = end != null ? end - start : start

  if (interval >= Math.pow(2, 32)) {
    throw Error(`Not implemented`)
  }

  const byteAmount = 32 - Math.clz32(interval - 1)

  let bytes = new Uint8Array(randomBytes(Math.max(byteAmount / 8, 1)))
  let bitCounter = 0

  function nextBit(): number {
    let result = bytes[0] % 2
    bytes[0] = bytes[0] >> 1
    if (++bitCounter == 8) {
      bitCounter = 0
      bytes = bytes.subarray(1)
    }
    return result
  }

  let result = 0
  for (let i = 0; i < byteAmount; i++) {
    if (result + (1 << i) < interval) {
      if (nextBit() == 1) {
        result |= 1 << i
      }
    }
  }

  return end != null ? start + result : result
}
