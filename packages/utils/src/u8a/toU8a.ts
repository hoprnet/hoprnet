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

  if (length <= 4 && arg > (1 << length * 8)) {
    throw Error(`Argument <${arg}> does not fit into desired length <${length}>.`)
  }

  if (arg == 0) {
    return new Uint8Array(length).fill(0)
  }

  const buf = new Uint8Array(4)

  const view = new DataView(buf.buffer, 0)

  view.setUint32(0, arg)

  if (length == undefined) {
    return buf
  } else if (length <= 4) {
    return buf.slice(4 - length)
  } else {
    let result = new Uint8Array(length)

    result.set(buf, length - 4)

    return result
  }
}

/**
 * Converts a **HEX** string to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @example
 * stringToU8a('0xDEadBeeF') // Uint8Array [ 222, 173, 190, 239 ]
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
