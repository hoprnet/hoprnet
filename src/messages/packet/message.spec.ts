import assert from 'assert'
import { PACKET_SIZE, MAX_HOPS } from '../../constants'
import Message, { PADDING } from './message'
import { u8aConcat, LENGTH_PREFIX_LENGTH } from '../../utils'
import { randomBytes } from 'crypto'
import { PRIVATE_KEY_LENGTH } from './header/parameters'
import secp256k1 from 'secp256k1'

describe('test class that encapsulates (encrypted and padded) messages', function() {
  it('should create a Message object and encrypt / decrypt it', function() {
    const msg = Message.createPlain('test')

    const testMessage = new TextEncoder().encode('test')

    assert(
      u8aConcat(
        new Uint8Array([0, 0, 0, 4]),
        PADDING,
        testMessage,
        new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH - testMessage.length)
      ).every((value: number, index: number) => value == msg[index])
    )

    assert.throws(() => Message.createPlain(new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH + 1)))

    const secrets = []

    for (let i = 0; i < 2; i++) {
      secrets.push(secp256k1.publicKeyCreate(randomBytes(PRIVATE_KEY_LENGTH)))
    }

    msg.onionEncrypt(secrets)

    secrets.forEach((secret: Uint8Array) => {
      msg.decrypt(secret)
    })

    msg.encrypted = false

    assert.deepEqual(msg.plaintext, testMessage)
  })
})
