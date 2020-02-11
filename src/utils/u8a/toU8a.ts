/**
 * Converts a number to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @param arg to convert to Uint8Array
 * @param length desired length of the Uint8Array
 */
export function toU8a(arg: number, length?: number): Uint8Array {
  if (!Number.isInteger(arg) || arg < 0) {
    throw Error('Not implemented')
  }

  let str = arg.toString(16)

  return stringToU8a(str, length)
}

/**
 * Converts a string to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @notice Throws an error in case a length was provided and the result does not fit.
 * @param str string to convert
 * @param length desired length of the Uint8Array
 */
export function stringToU8a(str: string, length?: number): Uint8Array {
  if (str.startsWith('0x')) {
    str = str.slice(2)
  }

  if (str.length % 2 == 1) {
    str = '0'.concat(str)
  }

  if (length != null && str.length >> 1 > length) {
    throw Error('Input argument has too many hex decimals.')
  }

  if (length != null && str.length >> 1 < length) {
    str = str.padStart(length << 1, '0')
  }

  return Uint8Array.from(str.match(/[0-9a-f]{2}/g).map(x => parseInt(x, 16)))
}