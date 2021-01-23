/**
 * This folder includes the encoders / decoders required to translate
 * our SC logs to events.
 */

import { u8aToHex } from '@hoprnet/hopr-utils'
import { Public } from '../../types'
import { getTopic0 } from './utils'
export * from './logs'
export * from './utils'
export * from './types'

/**
 * @TODO: requires documentantion
 * @param rawTopic
 * @param first
 * @param second
 * @param bidirectional
 */
export const generateTopics = (
  rawTopic: Uint8Array,
  first?: Public,
  second?: Public,
  bidirectional?: boolean
): (undefined | string | string[])[] => {
  if (bidirectional && (first == null || second == null)) {
    throw Error(`Bidirectional property can only be used if 'first' and 'second' are set`)
  }

  const topic0 = []

  if ((first == null && second == null) || (bidirectional && first[0] % 4 != second[0] % 4)) {
    for (let i = 0; i < 4; i++) {
      topic0.push(getTopic0(rawTopic, (i >> 1) % 2, i % 2))
    }
  } else if (first != null) {
    for (let i = 0; i < 2; i++) {
      topic0.push(getTopic0(rawTopic, first[0], i % 2))
    }
  } else if (second != null) {
    for (let i = 0; i < 2; i++) {
      topic0.push(getTopic0(rawTopic, (i >> 1) % 2, second[0]))
    }
  } else {
    topic0.push(getTopic0(rawTopic, first[0], second[0]))
  }

  if (bidirectional) {
    return [
      topic0,
      [u8aToHex(first.slice(1, 33)), u8aToHex(second.slice(1, 33))],
      [u8aToHex(first.slice(1, 33)), u8aToHex(second.slice(1, 33))]
    ]
  } else {
    return [
      topic0,
      first != null ? u8aToHex(first.slice(1, 33)) : undefined,
      second != null ? u8aToHex(second.slice(1, 33)) : undefined
    ]
  }
}
