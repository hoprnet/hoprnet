import {Message} from './message'
import assert from 'assert'

describe('test Message', () => {
  const from = '16Uiu2HAkyuTGEAywCMrGg4nv6sA37k2HtXb2NHBfoGzi2KrGVZo2'
  const text = 'hello world'
  // prettier-ignore
  const buffer = new Uint8Array([
    49,  54,  85, 105, 117,  50,  72,  65, 107, 121, 117,
    84,  71,  69,  65, 121, 119,  67,  77, 114,  71, 103,
    52, 110, 118,  54, 115,  65,  51,  55, 107,  50,  72,
    116,  88,  98,  50,  78,  72,  66, 102, 111,  71, 122,
    105,  50,  75, 114,  71,  86,  90, 111,  50, 58, 104,
    101,  108, 108, 111,  32, 119, 111, 114, 108, 100
  ])

  it('should initialize Message', () => {
    const message = Message.fromJson({
      from,
      text,
    })

    assert.deepEqual(message.toU8a(), buffer)
  })

  it('should initialize Message from buffer', () => {
    const message = new Message(buffer)

    assert.deepEqual(message.toU8a(), buffer)
  })

  it('Message json should be correct', () => {
    const message = Message.fromJson({
      from,
      text,
    })

    const parsed = message.toJson()

    assert(parsed.from === from)
    assert(parsed.text === text)
  })
})
