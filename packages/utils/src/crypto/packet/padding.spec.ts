import { addPadding, removePadding, PADDING_TAG, PADDING_TAG_LENGTH } from './padding'
import { u8aEquals } from '../../u8a'
import assert from 'assert'
import { PAYLOAD_SIZE } from './constants'

describe('padding', function () {
  it('test adding padding and removing it', function () {
    const TEST_MESSAGE = new TextEncoder().encode('text')

    const paddedMsg = addPadding(TEST_MESSAGE)

    assert(paddedMsg.length == PAYLOAD_SIZE)

    assert(u8aEquals(removePadding(paddedMsg), TEST_MESSAGE))
  })

  it('test adding padding and removing it - false positives', function () {
    const TEST_MESSAGE = new TextEncoder().encode('text')

    const paddedMsg = Uint8Array.from([
      ...new Uint8Array(PAYLOAD_SIZE - TEST_MESSAGE.length - PADDING_TAG_LENGTH),
      ...new TextEncoder().encode('H-PR'),
      ...TEST_MESSAGE
    ])

    assert.throws(() => removePadding(paddedMsg), Error(`Incorrect padding.`))

    const paddedMsg2 = addPadding(TEST_MESSAGE).slice(1)

    assert.throws(() => removePadding(paddedMsg2), Error(`Incorrect message length. Dropping message.`))

    const paddedMsg3 = Uint8Array.from([...new Uint8Array(PAYLOAD_SIZE - PADDING_TAG_LENGTH), ...PADDING_TAG])

    assert.throws(() => removePadding(paddedMsg3), Error(`Incorrect padding.`))

    const paddedMsg4 = Uint8Array.from([
      ...new Uint8Array(PAYLOAD_SIZE - PADDING_TAG_LENGTH + 1),
      ...PADDING_TAG.slice(1)
    ])

    assert.throws(() => removePadding(paddedMsg4), Error(`Incorrect padding.`))
  })
})
