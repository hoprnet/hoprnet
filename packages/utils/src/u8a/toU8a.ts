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

  return stringToU8a(arg.toString(16), length)
}

/**
 * Converts a string to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @notice Throws an error in case a length was provided and the result does not fit.
 * @param str string to convert
 * @param length desired length of the Uint8Array
 */
export function stringToU8a(str: string, length?: number): Uint8Array {
  if (length != null && length <= 0) {
    return new Uint8Array([])
  }

  if (str.startsWith('0x')) {
    str = str.slice(2)
  }

  let strLength = str.length

  if ((strLength & 1) == 1) {
    str = '0' + str
    strLength++
  }

  if (length != null && str.length >> 1 > length) {
    throw Error('Input argument has too many hex decimals.')
  }

  if (length != null && str.length >> 1 < length) {
    str = str.padStart(length << 1, '0')
    strLength = length << 1
  }

  const arr = new Uint8Array(strLength >> 1)

  for (let i = 0; i < strLength; i += 2) {
    const strSlice = str.slice(i, i + 2).match(/[0-9a-fA-F]{2}/g)

    if (strSlice == null || strSlice.length != 1) {
      throw Error(`Got unknown character '${str.slice(i, i + 2)}'`)
    }

    arr[i >> 1] = parseInt(strSlice[0], 16)
  }

  return arr
}
