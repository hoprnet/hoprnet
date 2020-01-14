import assert from 'assert'
import { PACKET_SIZE, MAX_HOPS } from '../../constants'
import Message, { PADDING } from './message'
import { u8aConcat, LENGTH_PREFIX_LENGTH } from '../../utils'
import {randomBytes} from 'crypto'
import { PRIVATE_KEY_LENGTH } from './header/parameters'
import secp256k1 from 'secp256k1'
import forEachRight from 'lodash.foreachright'

describe('test class that encapsulates (encrypted and padded) messages', function() {
  it('should create a Message object and encrypt / decrypt it', function() {
    const msg = Message.createPlain('test')

    const testMessage = new TextEncoder().encode('test')

    assert.deepEqual(msg, u8aConcat(new Uint8Array([0, 0, 0, 4]), PADDING, testMessage, new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH - testMessage.length)))

    assert.throws(() => Message.createPlain(new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH + 1)))

    const secrets = []

    for(let i = 0; i < 2; i++) {
      secrets.push(secp256k1.publicKeyCreate(randomBytes(PRIVATE_KEY_LENGTH)))
    }

    msg.onionEncrypt(secrets)

    secrets.forEach((secret: Uint8Array) => {
      msg.decrypt(secret)
    })

    msg.encrypted = false

    assert.deepEqual(msg.plaintext, testMessage)
  })

  //   it('should convert a length-prefixed u8a with additional padding to u8a', function() {
  //     assert.deepEqual(lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([1])), new Uint8Array([255]))

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([2])))

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 255]), new Uint8Array([2])))

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 255, 1]), new Uint8Array([1])))

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1]), new Uint8Array([1])))

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([0, 0, 0]), new Uint8Array([1])))

  //     assert.deepEqual(
  //       lengthPrefixedToU8a(toLengthPrefixedU8a(new Uint8Array([1, 2, 3, 4]), new Uint8Array([1])), new Uint8Array([1])),
  //       new Uint8Array([1, 2, 3, 4])
  //     )

  //     assert.throws(() => lengthPrefixedToU8a(new Uint8Array([]), new Uint8Array([1]), 2))

  //     assert.deepEqual(lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 1, 0]), null, 7), new Uint8Array([1])), new Uint8Array([1])
  //   })
})
