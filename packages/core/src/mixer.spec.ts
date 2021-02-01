import type { Packet } from './messages/packet'
import { Mixer } from './mixer'
import assert from 'assert'
import { MAX_PACKET_DELAY } from './constants'

let i = 0
let fakePacket = () => {
  return (i++ as unknown) as Packet<any>
}

describe('test mixer ', function () {
  it('should push and pop one element', async function () {
    const m = new Mixer()

    const p1 = fakePacket()
    assert(m.length == 0, `mixer must be empty before pushing`)
    m.push(p1)

    const it = m[Symbol.asyncIterator]()

    const before = Date.now()

    assert((await it.next()).value == p1, 'First packet must be packet number 1')

    // 10 ms propagation timeout
    const PROPAGATION_OFFSET = 10
    assert(Date.now() - before < MAX_PACKET_DELAY + PROPAGATION_OFFSET, `Delay must not be longer than max delay`)

    assert(m.length == 0)
  })

  it('should push and pop multiple elements', async function () {
    const m = new Mixer()

    const AMOUNT_OF_PACKETS = 4
    const packets = Array.from({ length: AMOUNT_OF_PACKETS }, (_) => fakePacket())

    assert(!m.notEmpty(), 'Mixer must be empty')

    packets.forEach((p) => m.push(p))

    assert(m.length == AMOUNT_OF_PACKETS)

    const it = m[Symbol.asyncIterator]()

    const receivedPackets: Packet<any>[] = []

    for (let i = 0; i < AMOUNT_OF_PACKETS; i++) {
      receivedPackets.push((await it.next()).value)
    }

    receivedPackets.sort()

    for (const [index, packet] of packets.entries()) {
      assert(receivedPackets[index] == packet, `Mixer must return all packages`)
    }

    assert(!m.notEmpty(), 'Mixer must be empty')
  })

  it('should push elements, drain the mixer, push new elemnts and pop elements', async function () {
    const m = new Mixer()

    const AMOUNT_OF_PACKETS = 4
    const packets = Array.from({ length: AMOUNT_OF_PACKETS }, (_) => fakePacket())

    packets.forEach((p) => m.push(p))

    const it = m[Symbol.asyncIterator]()

    const receivedPackets: Packet<any>[] = []

    for (let i = 0; i < AMOUNT_OF_PACKETS; i++) {
      receivedPackets.push((await it.next()).value)
    }

    const newPacket = fakePacket()

    const beforeRunningEmpty = Date.now()

    const REFUND_TIMEOUT = 200

    setTimeout(() => {
      m.push(newPacket)
    }, REFUND_TIMEOUT)

    const waitPromise = it.next()

    assert(!m.notEmpty(), 'Queue must be empty')

    const nextPacket = (await waitPromise).value

    assert(nextPacket == newPacket)
    assert(Date.now() - beforeRunningEmpty >= REFUND_TIMEOUT, 'Should not return a packet before it got new packets')
  })

  it('should come to an end', async function () {
    const m = new Mixer()

    assert(!m.done)

    const it = m[Symbol.asyncIterator]()

    const beforeEnding = Date.now()
    const END_TIMEOUT = 200
    setTimeout(() => {
      m.end()
    }, END_TIMEOUT)

    const returnValue = await it.next()

    assert(returnValue.value == undefined, `Last message should be undefined`)

    assert(Date.now() - beforeEnding >= END_TIMEOUT)
  })

  it('should come to an end even if packets are waiting', async function () {
    const m = new Mixer()

    assert(!m.done)

    const it = m[Symbol.asyncIterator]()

    const beforeEnding = Date.now()

    const END_TIMEOUT = 1
    setTimeout(() => {
      m.end()
    }, END_TIMEOUT)

    // Drain the iterator
    while (true) {
      const result = await it.next()

      if (result.done) {
        break
      }
    }

    assert(Date.now() - beforeEnding >= END_TIMEOUT, `Should not timeout before `)
  })
  // it('should push and pop multiple element', async function () {
  //   const m = new Mixer(fakeIncrementer)
  //   const p1 = fakePacket()
  //   const p2 = fakePacket()
  //   const p3 = fakePacket()
  //   const p4 = fakePacket()

  //   assert(m.poppable() === false, 'empty mixer is not poppable')
  //   m.push(p1)
  //   m.push(p2)
  //   m.push(p3)
  //   assert(m.poppable() === false, 'packets are not immediately poppable')
  //   time += MAX_PACKET_DELAY + 1
  //   m.push(p4)
  //   assert(m.poppable() === true, 'packets are poppable after max delay')
  //   m.pop()
  //   assert(m.poppable() === true, 'packets are poppable after popped')
  //   m.pop()
  //   assert(m.poppable() === true, 'packets are poppable after popped 2')
  //   m.pop()
  //   assert(m.poppable() === false, 'all due packets popped')
  //   time += MAX_PACKET_DELAY + 1
  //   assert(m.poppable() === true, 'final packet poppable')
  //   m.pop()
  //   assert(m.poppable() === false, 'all due packets popped 2')
  // })

  // it('probabilistic test, packet ordering', async function () {
  //   const m = new Mixer(fakeIncrementer)
  //   for (let x = 0; x < 1000; x++) {
  //     m.push(fakePacket())
  //   }
  //   assert(m.poppable() === false, 'packets are not immediately poppable')
  //   time += MAX_PACKET_DELAY + 1
  //   assert(m.poppable() === true, 'packets are poppable after max delay')
  //   let ordered = true
  //   let prev = 0
  //   for (let x = 0; x < 1000; x++) {
  //     assert(m.poppable(), 'should be poppable after ' + x)
  //     let next = (m.pop() as unknown) as number // cast back to fake
  //     if (next <= prev) {
  //       ordered = false
  //     }
  //     prev = next
  //   }
  //   assert(!ordered, 'packets should be shuffled')
  // })
})
