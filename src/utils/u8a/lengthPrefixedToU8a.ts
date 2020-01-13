const LENGTH_PREFIX_LENGTH = 4

/**
 * Decodes a length-prefixed array and returns the encoded data.
 *
 * @param arg array to decode
 * @param additionalPadding additional padding to remove
 */
export function lengthPrefixedToU8a(arg: Uint8Array, additionalPadding?: Uint8Array) {
  if (arg.length < LENGTH_PREFIX_LENGTH || (additionalPadding != null && arg.length < LENGTH_PREFIX_LENGTH + additionalPadding.length)) {
    throw Error(
      `Expected a ${Uint8Array.name} of at least length ${
        additionalPadding != null ? LENGTH_PREFIX_LENGTH + additionalPadding.length : LENGTH_PREFIX_LENGTH
      } but got ${arg.length}.`
    )
  }

  let length = parseInt(
    arg.subarray(0, LENGTH_PREFIX_LENGTH).reduce((acc, n) => (acc += n.toString(16).padStart(2, '0')), ''),
    16
  )

  if (!Number.isInteger(length)) {
    throw Error(`Invalid encoded length.`)
  }

  if (additionalPadding != null ? length + additionalPadding.length + LENGTH_PREFIX_LENGTH != arg.length : length + LENGTH_PREFIX_LENGTH != arg.length) {
    throw Error(
      `Invalid array length. Expected a ${Uint8Array.name} of at least length ${
        additionalPadding != null ? LENGTH_PREFIX_LENGTH + additionalPadding.length + length : LENGTH_PREFIX_LENGTH + length
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
    return arg.subarray(LENGTH_PREFIX_LENGTH + additionalPadding.length)
  } else {
    return arg.subarray(LENGTH_PREFIX_LENGTH)
  }
}
