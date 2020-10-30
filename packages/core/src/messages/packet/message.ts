import {PACKET_SIZE} from '../../constants'
import {deriveCipherParameters} from './header'

import {toLengthPrefixedU8a, lengthPrefixedToU8a, PRP} from '@hoprnet/hopr-utils'

export const PADDING = new TextEncoder().encode('PADDING')

class Message extends Uint8Array {
  public encrypted: boolean
  constructor(
    _encrypted: boolean,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ) {
    if (arr == null) {
      super(Message.SIZE)
    } else {
      super(arr.bytes, arr.offset, PACKET_SIZE)
    }

    this.encrypted = _encrypted
  }

  static get SIZE(): number {
    return PACKET_SIZE
  }

  slice(begin: number = 0, end: number = Message.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = Message.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  getCopy(): Message {
    const msgCopy = new Message(this.encrypted)

    msgCopy.set(this)

    return msgCopy
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

  static create(
    msg: Uint8Array,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Message {
    const message = new Message(false, arr)

    message.set(toLengthPrefixedU8a(msg, PADDING, PACKET_SIZE))

    return message
  }

  onionEncrypt(secrets: Uint8Array[]): Message {
    if (!Array.isArray(secrets) || secrets.length <= 0) {
      throw Error('Invald input arguments. Expected array with at least one secret key.')
    }

    this.encrypted = true

    for (let i = secrets.length; i > 0; i--) {
      const {key, iv} = deriveCipherParameters(secrets[i - 1])

      PRP.createPRP(key, iv).permutate(this)
    }

    return this
  }

  decrypt(secret: Uint8Array): Message {
    const {key, iv} = deriveCipherParameters(secret)

    PRP.createPRP(key, iv).inverse(this)

    return this
  }
}

export default Message
