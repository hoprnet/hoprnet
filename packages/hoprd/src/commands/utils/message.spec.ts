import { decodeMessage, encodeMessage } from './message.js'
import assert from 'assert'

describe('Message encoding & decoding', () => {
  it('check if message can be encoded and then decoded', async () => {
    let msg = "some test message!"
    let encodedMsg = encodeMessage(msg)

    await new Promise(r => setTimeout(r, 2000));

    let decodedMsg = decodeMessage(encodedMsg)
    assert.deepEqual(decodedMsg.msg, msg)
    assert(decodedMsg.latency >= 2000)
  })
})