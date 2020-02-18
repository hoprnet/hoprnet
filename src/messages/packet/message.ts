import { PACKET_SIZE } from '../../constants'
import { deriveCipherParameters } from './header'

import { PRP, toLengthPrefixedU8a, lengthPrefixedToU8a } from '../../utils'

export const PADDING = new TextEncoder().encode('PADDING')

export default class Message extends Uint8Array {
  constructor(
    arr: {
      bytes: ArrayBuffer
      offset: number
    },
    public encrypted: boolean
  ) {
    super(arr.bytes, arr.offset, PACKET_SIZE)
  }

  static get SIZE(): number {
    return PACKET_SIZE
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, this.byteOffset + begin, end != null ? end - begin : undefined)
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
    return new Message(
      {
        bytes: msg.buffer,
        offset: 0
      },
      true
    )
  }

  static createPlain(msg: Uint8Array | string): Message {
    if (typeof msg == 'string') {
      msg = new TextEncoder().encode(msg)
    }

    return new Message(
      {
        bytes: toLengthPrefixedU8a(msg, PADDING, PACKET_SIZE),
        offset: 0
      },
      false
    )
  }

  onionEncrypt(secrets: Uint8Array[]): Message {
    if (!Array.isArray(secrets) || secrets.length <= 0) {
      throw Error('Invald input arguments. Expected array with at least one secret key.')
    }

    this.encrypted = true

    for (let i = secrets.length; i > 0; i--) {
      const { key, iv } = deriveCipherParameters(secrets[i - 1])

      PRP.createPRP(key, iv).permutate(this.subarray())
    }

    return this
  }

  decrypt(secret: Uint8Array): Message {
    const { key, iv } = deriveCipherParameters(secret)

    PRP.createPRP(key, iv).inverse(this.subarray())

    return this
  }
}
