import forEachRight from 'lodash.foreachright'

import { PACKET_SIZE } from '../../constants'
import { deriveCipherParameters } from './header'

import { PRP, toLengthPrefixedU8a, lengthPrefixedToU8a } from '../../utils'

export const PADDING = new TextEncoder().encode('PADDING')

export default class Message extends Uint8Array {
  private constructor(arr: Uint8Array, public encrypted: boolean) {
    super(arr)

    if (arr.length != PACKET_SIZE) {
      throw Error(`Expected a ${Uint8Array.name} of size ${PACKET_SIZE} but got ${arr.length}.`)
    }
  }

  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }


  get plaintext(): Uint8Array {
    if (this.encrypted) {
      throw Error(`Cannot read encrypted data.`)
    }

    return lengthPrefixedToU8a(this, PADDING, PACKET_SIZE)
  }

  get ciphertext(): Uint8Array {
    if (!this.encrypted) {
      throw Error(`Message is unencrypted. Cannot read encrypted data.`)
    }

    return this
  }

  static createEncrypted(msg: Uint8Array): Message {
    return new Message(msg, true)
  }

  static createPlain(msg: Uint8Array | string): Message {
    if (typeof msg == 'string') {
      msg = new TextEncoder().encode(msg)
    }

    return new Message(toLengthPrefixedU8a(msg, PADDING, PACKET_SIZE), false)
  }

  onionEncrypt(secrets: Uint8Array[]): Message {
    if (!Array.isArray(secrets) || secrets.length <= 0) {
      throw Error('Invald input arguments. Expected array with at least one secret key.')
    }

    this.encrypted = true

    forEachRight(secrets, (secret: Uint8Array) => {
      const { key, iv } = deriveCipherParameters(secret)
      PRP.createPRP(key, iv).permutate(this)
    })

    return this
  }

  decrypt(secret: Uint8Array): Message {
    const { key, iv } = deriveCipherParameters(secret)

    PRP.createPRP(key, iv).inverse(this)

    return this
  }
}

export const SIZE = PACKET_SIZE
