import { toU8a, u8aToNumber } from '@hoprnet/hopr-utils'

export const LENGTH_PREFIX_SIZE = 4

export function encodeWithLengthPrefix(msg: Uint8Array): Uint8Array {
  if (msg.length > 2 ** (LENGTH_PREFIX_SIZE * 8) - 1) {
    throw Error(`Message length does not fit into ${LENGTH_PREFIX_SIZE} bytes`)
  }

  return Uint8Array.from([...toU8a(msg.length, LENGTH_PREFIX_SIZE), ...msg])
}

export function decodeWithLengthPrefix(msg: Uint8Array): Uint8Array[] {
  const result: Uint8Array[] = []
  if (msg.length < LENGTH_PREFIX_SIZE) {
    throw Error(
      `Unable to read message because given array (size ${msg.length} elements) is too small to include length prefix.`
    )
  }

  let index = 0
  while (index < msg.length) {
    const length = u8aToNumber(msg.slice(index, index + LENGTH_PREFIX_SIZE)) as number

    index += LENGTH_PREFIX_SIZE
    if (index + length > msg.length) {
      throw Error(`Invalid length prefix encoding. Encoded length does not fit to given array`)
    }

    result.push(msg.slice(index, index + length))

    index += length
  }

  return result
}
