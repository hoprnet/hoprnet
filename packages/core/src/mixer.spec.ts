import type { Packet } from './messages/packet'
import { Mixer } from './mixer'
import assert from 'assert'

let i = 0
let fakePacket = () => {
  return i++ as unknown as Packet<any>
}

describe('test mixer ', function () {
  it('should push and pop a single element', async function () {
    const m = new Mixer()
    const p1 = fakePacket()
    assert(m.poppable() === false, 'empty mixer is not poppable')
    m.push(p1)
    let pOut = m.pop()
    assert(p1 === pOut)
  })
})
