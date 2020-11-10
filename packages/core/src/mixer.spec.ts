import type { Packet } from './messages/packet'
import { Mixer } from './mixer'
import assert from 'assert'
import { MAX_PACKET_DELAY } from './constants'

let i = 0
let fakePacket = () => {
  return (i++ as unknown) as Packet<any>
}

let time = 0
let fakeIncrementer = () => {
  return time
}

describe('test mixer ', function () {
  it('should push and pop a single element', async function () {
    const m = new Mixer(fakeIncrementer)
    const p1 = fakePacket()
    assert(m.poppable() === false, 'empty mixer is not poppable')
    m.push(p1)
    assert(m.poppable() === false, 'packets are not immediately poppable')
    time += MAX_PACKET_DELAY + 1
    assert(m.poppable() === true, 'packets are poppable after max delay')
    let pOut = m.pop()
    assert(p1 === pOut)
  })

  it('should push and pop multiple element', async function () {
    const m = new Mixer(fakeIncrementer)
    const p1 = fakePacket()
    const p2 = fakePacket()
    const p3 = fakePacket()
    const p4 = fakePacket()

    assert(m.poppable() === false, 'empty mixer is not poppable')
    m.push(p1)
    m.push(p2)
    m.push(p3)
    assert(m.poppable() === false, 'packets are not immediately poppable')
    time += MAX_PACKET_DELAY + 1
    m.push(p4)
    assert(m.poppable() === true, 'packets are poppable after max delay')
    m.pop()
    assert(m.poppable() === true, 'packets are poppable after popped')
    m.pop()
    assert(m.poppable() === true, 'packets are poppable after popped 2')
    m.pop()
    assert(m.poppable() === false, 'all due packets popped')
    time += MAX_PACKET_DELAY + 1
    assert(m.poppable() === true, 'final packet poppable')
    m.pop()
    assert(m.poppable() === false, 'all due packets popped 2')
  })

  it('probabilistic test, packet ordering', async function () {
    const m = new Mixer(fakeIncrementer)
    for (let x = 0; x < 1000; x++) {
      m.push(fakePacket())
    }
    assert(m.poppable() === false, 'packets are not immediately poppable')
    time += MAX_PACKET_DELAY + 1
    assert(m.poppable() === true, 'packets are poppable after max delay')
    let ordered = true
    let prev = 0
    for (let x = 0; x < 1000; x++) {
      assert(m.poppable(), 'should be poppable after ' + x)
      let next = (m.pop() as unknown) as number // cast back to fake
      if (next <= prev) {
        ordered = false
      }
      prev = next
    }
    assert(!ordered, 'packets should be shuffled')
  })
})
