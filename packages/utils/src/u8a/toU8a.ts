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

  if (length && length < 4 && arg > 1 << (length * 8)) {
    throw Error(`Argument <${arg}> does not fit into desired length <${length}>.`)
  }

  if (arg == 0) {
    return new Uint8Array(length ?? 1).fill(0)
  }

  const buf = new Uint8Array(4)

  const view = new DataView(buf.buffer, 0)

  view.setUint32(0, arg)

  if (length == undefined) {
    if (buf[0]) {
      return buf
    }

    if (buf[1]) {
      return buf.subarray(1)
    }

    if (buf[2]) {
      return buf.subarray(2)
    }

    return buf.subarray(3)
  } else if (length <= 4) {
    return buf.subarray(4 - length)
  } else {
    let result = new Uint8Array(length)

    result.set(buf, length - 4)

    return result
  }
}

function lookup(str: string): 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 {
  switch (str.charCodeAt(0)) {
    case 48:
      return 0
    case 49:
      return 1
    case 50:
      return 2
    case 51:
      return 3
    case 52:
      return 4
    case 53:
      return 5
    case 54:
      return 6
    case 55:
      return 7
    case 56:
      return 8
    case 57:
      return 9
    case 65:
    case 97:
      return 10
    case 66:
    case 98:
      return 11
    case 67:
    case 99:
      return 12
    case 68:
    case 100:
      return 13
    case 69:
    case 101:
      return 14
    case 70:
    case 102:
      return 15
    default:
      throw Error(`Got unknown hex character '${str.substring(0, 1)}'`)
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
    str = str.substring(2)
  }

  let strLength = str.length

  if ((strLength & 1) == 1) {
    str = '0' + str
    strLength++
  }

  if (length != null && strLength >> 1 > length) {
    throw Error('Input argument has too many hex decimals.')
  }

  if (length != null && strLength >> 1 < length) {
    str = str.padStart(length << 1, '0')
    strLength = length << 1
  }

  const arr = new Uint8Array(strLength >> 1)

  for (let i = 0; i < strLength; i += 2) {
    arr[i >> 1] = (lookup(str.substring(i, i + 1)) << 4) + lookup(str.substring(i + 1, i + 2))
  }

  return arr
}
