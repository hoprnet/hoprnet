import type { Packet } from './messages/index.js'
import { Mixer } from './mixer.js'
import assert from 'assert'

class TestingMixer extends Mixer {
  addPacket(priority: number, packet: Packet) {
    this.queue.push([priority, packet])
  }

  emitPacket() {
    this.timer.reschedule(1)
  }
}
let i = 0
let fakePacket = () => {
  return i++ as unknown as Packet
}

describe('test mixer ', async function () {
  it('add one packet and emit one packet', async function () {
    const mixer = new TestingMixer()

    const firstPacket = fakePacket()
    mixer.push(firstPacket)

    mixer.emitPacket()

    const packets = []

    const it = mixer[Symbol.asyncIterator]()
    packets.push((await it.next()).value)

    assert(packets.length == 1)
    assert(packets[0] == firstPacket)
  })

  it('add packets then drain them', async function () {
    const mixer = new TestingMixer(() => 50)

    const firstPacket = fakePacket()
    mixer.push(firstPacket)
    const secondPacket = fakePacket()
    mixer.push(secondPacket)

    mixer.emitPacket()

    const packets = []

    const it = mixer[Symbol.asyncIterator]()

    packets.push((await it.next()).value)
    mixer.emitPacket()

    packets.push((await it.next()).value)

    assert(packets.length == 2)
    assert(packets[0] == firstPacket)
    assert(packets[1] == secondPacket)
  })

  it('add and drain packets interleaved', async function () {
    const mixer = new TestingMixer(() => 50)

    const firstPacket = fakePacket()
    mixer.push(firstPacket)

    mixer.emitPacket()

    const it = mixer[Symbol.asyncIterator]()
    const packets = []

    packets.push((await it.next()).value)

    const secondPacket = fakePacket()
    mixer.push(secondPacket)

    packets.push((await it.next()).value)

    assert(packets.length == 2)
    assert(packets[0] == firstPacket)
    assert(packets[1] == secondPacket)
  })

  it('reorder packets', async function () {
    let callCount = 0
    const mixer = new TestingMixer(() => {
      switch (callCount++) {
        case 0:
          return 100
        case 1:
          return 1
        default:
          throw Error(`should not happen`)
      }
    })

    const firstPacket = fakePacket()
    mixer.push(firstPacket)
    const secondPacket = fakePacket()
    mixer.push(secondPacket)

    const it = mixer[Symbol.asyncIterator]()
    const packets = []

    packets.push((await it.next()).value)
    packets.push((await it.next()).value)

    assert(packets.length == 2)
    assert(packets[0] == secondPacket)
    assert(packets[1] == firstPacket)
  })

  it('end empty mixer', async function () {
    const mixer = new TestingMixer()

    setTimeout(mixer.end, 100)

    for await (const _msg of mixer) {
      assert.fail(`Must not emit a msg`)
    }

    // Produces a timeout if not succesful
  })

  it('end filled mixer', async function () {
    const mixer = new TestingMixer(() => 100)

    const firstPacket = fakePacket()
    mixer.push(firstPacket)

    setTimeout(mixer.end, 50)

    for await (const _msg of mixer) {
      assert.fail(`Must not emit a msg`)
    }

    // Produces a timeout if not succesful
  })
})
