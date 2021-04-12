import { PAYLOAD_SIZE } from './constants'
import { u8aEquals } from '../../u8a'

export const PADDING_TAG = Uint8Array.from([72, 79, 80, 82])
export const PADDING_TAG_LENGTH = 4

export function addPadding(msg: Uint8Array) {
  const msgLength = msg.length

  if (msgLength > PAYLOAD_SIZE - PADDING_TAG_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return Uint8Array.from([...new Uint8Array(PAYLOAD_SIZE - PADDING_TAG_LENGTH - msgLength), ...PADDING_TAG, ...msg])
}

export function removePadding(decoded: Uint8Array) {
  if (decoded.length != PAYLOAD_SIZE) {
    throw Error(`General error.`)
  }

  const index = decoded.indexOf(PADDING_TAG[0])

  if (index < 0 || index >= PAYLOAD_SIZE - PADDING_TAG_LENGTH || !u8aEquals(decoded.subarray(index, index + PADDING_TAG_LENGTH), PADDING_TAG)) {
    throw new Error(`General error.`)
  }

  return decoded.slice(index + PADDING_TAG_LENGTH)
}
