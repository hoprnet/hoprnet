import assert from 'assert'
import { PACKET_SIZE } from '../../constants'
import Message, { PADDING } from './message'
import { u8aConcat, u8aEquals, LENGTH_PREFIX_LENGTH } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'
import { PRIVATE_KEY_LENGTH } from './header/parameters'
import secp256k1 from 'secp256k1'

describe('test messages', function () {
  it('should create a Message object and encrypt / decrypt it', function () {
    const testMessage = new TextEncoder().encode('test')

    const msg = Message.create(testMessage)

    assert(
      u8aEquals(
        u8aConcat(
          new Uint8Array([0, 0, 0, 4]),
          PADDING,
          testMessage,
          new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH - testMessage.length)
        ),
        msg
      )
    )

    assert.throws(() => Message.create(new Uint8Array(PACKET_SIZE - PADDING.length - LENGTH_PREFIX_LENGTH + 1)))

    const secrets = []

    let msgCopy: Message
    for (let i = 0; i < 2; i++) {
      secrets.push(secp256k1.publicKeyCreate(randomBytes(PRIVATE_KEY_LENGTH)))

      msgCopy = msg.getCopy()

      msgCopy.onionEncrypt(secrets)

      secrets.forEach((secret: Uint8Array) => {
        msgCopy.decrypt(secret)
      })

      msgCopy.encrypted = false

      assert(u8aEquals(msgCopy.plaintext, testMessage))
    }
  })
})
