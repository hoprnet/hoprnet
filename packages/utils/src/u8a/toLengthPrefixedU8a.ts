import { toU8a } from './toU8a'
import { u8aConcat } from './concat'

import { LENGTH_PREFIX_LENGTH } from '.'
/**
 * Adds a length-prefix to a Uint8Array
 * @param arg data to add padding
 * @param additionalPadding optional additional padding that is inserted between length and data
 * @param length optional target length
 *
 */
export function toLengthPrefixedU8a(arg: Uint8Array, additionalPadding?: Uint8Array, length?: number) {
  if (additionalPadding != null) {
    if (length != null && arg.length + LENGTH_PREFIX_LENGTH + additionalPadding.length > length) {
      throw Error(
        `Cannot create length-prefixed ${Uint8Array.name} because encoded ${Uint8Array.name} would be ${
          length - arg.length + LENGTH_PREFIX_LENGTH + additionalPadding.length
        } bytes greater than desired size of ${length} bytes.`
      )
    }

    if (length != null) {
      return u8aConcat(
        toU8a(arg.length, LENGTH_PREFIX_LENGTH),
        additionalPadding,
        arg,
        new Uint8Array(length - LENGTH_PREFIX_LENGTH - additionalPadding.length - arg.length)
      )
    } else {
      return u8aConcat(toU8a(arg.length, LENGTH_PREFIX_LENGTH), additionalPadding, arg)
    }
  } else {
    if (length != null && arg.length + LENGTH_PREFIX_LENGTH > length) {
      throw Error(
        `Cannot create length-prefixed ${Uint8Array.name} because encoded ${Uint8Array.name} would be ${
          length - arg.length + LENGTH_PREFIX_LENGTH
        } bytes greater than desired size of ${length} bytes.`
      )
    }

    if (length != null) {
      return u8aConcat(
        toU8a(arg.length, LENGTH_PREFIX_LENGTH),
        arg,
        new Uint8Array(length - LENGTH_PREFIX_LENGTH - arg.length)
      )
    } else {
      return u8aConcat(toU8a(arg.length, LENGTH_PREFIX_LENGTH), arg)
    }
  }
}
