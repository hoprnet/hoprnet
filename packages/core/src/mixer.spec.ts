import type { Packet } from './messages/packet'
import { Mixer } from './mixer'
import assert from 'assert'
import { MAX_PACKET_DELAY } from './constants'
import sinon from 'sinon'

let i = 0
let fakePacket = () => {
  return (i++ as unknown) as Packet<any>
}

describe('test mixer ', async function () {
  let clock: any

  beforeEach(async () => {
    clock = sinon.useFakeTimers(Date.now())
  })

  afterEach(() => {
    clock.restore()
  })

  it('should push and pop a single element', async function () {
    let calls = 0
    let lastCall = null
    const m = new Mixer((p) => {
      calls++
      lastCall = p
    })
    const p1 = fakePacket()
    assert.equal(calls, 0, 'empty mixer not delivered')
    m.push(p1)
    assert.equal(calls, 0, 'empty mixer not delivered immediately')
    await clock.tickAsync(MAX_PACKET_DELAY + 1)
    assert.equal(calls, 1, 'packets pop after max delay')
    assert(p1 === lastCall)
  })

  it('should push and pop multiple element', async function () {
    var calls = 0
    let lastCall = null
    const m = new Mixer((p) => {
      calls++
      lastCall = p
    })
    const p1 = fakePacket()
    const p2 = fakePacket()
    const p3 = fakePacket()
    const p4 = fakePacket()

    assert(calls === 0, 'empty mixer not delivered')
    m.push(p1)
    m.push(p2)
    m.push(p3)
    assert(calls === 0, 'empty mixer not delivered immediately')
    await clock.tickAsync(MAX_PACKET_DELAY + 1)
    m.push(p4)
    assert.equal(calls, 3, 'packets pop after max delay')
    await clock.tickAsync(MAX_PACKET_DELAY + 1)
    assert.equal(calls, 4, 'final packet poppable')
    assert(lastCall === p4, 'final packet is p4')
  })

  it('probabilistic test, packet ordering', async function () {
    var calls = 0
    let out: any[] = []
    const m = new Mixer((p) => {
      calls = calls + 1
      out.push(p)
    })
    for (let x = 0; x < 1000; x++) {
      m.push(fakePacket())
    }
    assert(calls === 0, 'empty mixer not delivered')
    await clock.tickAsync(MAX_PACKET_DELAY + 1)
    assert.equal(calls, 1000, '1000 messages delivered')
    let ordered = true
    let prev = 0
    for (let x = 0; x < 1000; x++) {
      let next = (out.pop() as unknown) as number // cast back to fake
      if (next <= prev) {
        ordered = false
      }
      prev = next
    }
    assert(!ordered, 'packets should be shuffled')
  })
})
