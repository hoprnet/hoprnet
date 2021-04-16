import { encode, decode } from 'rlp'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { Logger } from '@hoprnet/hopr-utils'

const log: Logger = Logger.getLogger('hoprd.utils.message')

/**
 * Adds the current timestamp to the message in order to measure the latency.
 * @param msg the message
 */
export function encodeMessage(msg: string): Uint8Array {
  return encode([msg, Date.now()])
}

/**
 * Tries to decode the message and returns the message as well as
 * the measured latency.
 * @param encoded an encoded message
 */
export function decodeMessage(
  encoded: Uint8Array
): {
  latency: number
  msg: string
} {
  let msg: Buffer, time: Buffer
  try {
    ;[msg, time] = decode(encoded) as [Buffer, Buffer]

    return {
      latency: Date.now() - parseInt(time.toString('hex'), 16),
      msg: msg.toString()
    }
  } catch (err) {
    log.error(`Could not decode received message '${u8aToHex(encoded)}'`, err)
    return {
      latency: NaN,
      msg: 'Error: Could not decode message'
    }
  }
}
