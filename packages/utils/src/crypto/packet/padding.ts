import { PAYLOAD_SIZE } from './constants'
import { u8aEquals } from '../../u8a'

export const PADDING_TAG = Uint8Array.from([72, 79, 80, 82]) // "HOPR"
export const PADDING_TAG_LENGTH = 4

/**
 * Adds a deterministic padding to a given payload.
 * @dev payloads that do not include the correct padding are
 * considered invalid
 * @param msg the payload to pad
 * @returns the padded payload
 */
export function addPadding(msg: Uint8Array) {
  const msgLength = msg.length

  if (msgLength > PAYLOAD_SIZE - PADDING_TAG_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return Uint8Array.from([...new Uint8Array(PAYLOAD_SIZE - PADDING_TAG_LENGTH - msgLength), ...PADDING_TAG, ...msg])
}

/**
 * Removes the padding from a given payload and fails if
 * the padding does not exist or if the payload has the
 * wrong size.
 * @param decoded a padded payload
 * @returns the message without the padding
 */
export function removePadding(decoded: Uint8Array) {
  if (decoded.length != PAYLOAD_SIZE) {
    throw Error(`Incorrect message length. Dropping message.`)
  }

  const index = decoded.indexOf(PADDING_TAG[0])

  if (
    index < 0 ||
    index >= PAYLOAD_SIZE - PADDING_TAG_LENGTH ||
    !u8aEquals(decoded.subarray(index, index + PADDING_TAG_LENGTH), PADDING_TAG)
  ) {
    throw Error(`Incorrect padding.`)
  }

  return decoded.slice(index + PADDING_TAG_LENGTH)
}
