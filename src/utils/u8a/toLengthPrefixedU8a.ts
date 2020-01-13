import { toU8a } from './toU8a'
import { u8aConcat } from './concat'

const LENGTH_PREFIX_LENGTH = 4
/**
 * Adds a length-prefix to a Uint8Array
 * @param arg data to add padding
 * @param additionalPadding optional additional padding that is inserted between
 * length and data
 */
export function toLengthPrefixedU8a(arg: Uint8Array, additionalPadding?: Uint8Array) {
  if (additionalPadding != null) {
    return u8aConcat(toU8a(arg.length, LENGTH_PREFIX_LENGTH), additionalPadding, arg)
  } else {
    return u8aConcat(toU8a(arg.length, LENGTH_PREFIX_LENGTH), arg)
  }
}
