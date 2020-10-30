import {LENGTH_PREFIX_LENGTH} from './constants'
import {u8aToNumber} from './u8aToNumber'

/**
 * Decodes a length-prefixed array and returns the encoded data.
 *
 * @param arg array to decode
 * @param additionalPadding additional padding to remove
 * @param targetLength optional target length
 */
export function lengthPrefixedToU8a(arg: Uint8Array, additionalPadding?: Uint8Array, targetLength?: number) {
  if (targetLength != null && arg.length < targetLength) {
    throw Error(`Expected a ${Uint8Array.name} of at least lenght ${targetLength}`)
  } else if (
    arg.length < LENGTH_PREFIX_LENGTH ||
    (additionalPadding != null && arg.length < LENGTH_PREFIX_LENGTH + additionalPadding.length)
  ) {
    throw Error(
      `Expected a ${Uint8Array.name} of at least length ${
        additionalPadding != null ? LENGTH_PREFIX_LENGTH + additionalPadding.length : LENGTH_PREFIX_LENGTH
      } but got ${arg.length}.`
    )
  }

  let arrLength = u8aToNumber(arg.subarray(0, LENGTH_PREFIX_LENGTH))

  if (!Number.isInteger(arrLength)) {
    throw Error(`Invalid encoded length.`)
  }

  if (
    targetLength == null &&
    (additionalPadding != null
      ? arrLength + additionalPadding.length + LENGTH_PREFIX_LENGTH != arg.length
      : arrLength + LENGTH_PREFIX_LENGTH != arg.length)
  ) {
    throw Error(
      `Invalid array length. Expected a ${Uint8Array.name} of at least length ${
        additionalPadding != null
          ? LENGTH_PREFIX_LENGTH + additionalPadding.length + arrLength
          : LENGTH_PREFIX_LENGTH + arrLength
      } but got ${arg.length}.`
    )
  }

  if (
    additionalPadding != null &&
    arg
      .subarray(LENGTH_PREFIX_LENGTH, LENGTH_PREFIX_LENGTH + additionalPadding.length)
      .some((value: number, index: number) => value != additionalPadding[index])
  ) {
    throw Error(`Array does not contain correct additional padding.`)
  }

  if (additionalPadding != null) {
    return arg.subarray(
      LENGTH_PREFIX_LENGTH + additionalPadding.length,
      LENGTH_PREFIX_LENGTH + additionalPadding.length + arrLength
    )
  } else {
    return arg.subarray(LENGTH_PREFIX_LENGTH, LENGTH_PREFIX_LENGTH + arrLength)
  }
}
