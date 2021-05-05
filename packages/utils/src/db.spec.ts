import { HoprDB } from './db'
import { randomBytes } from 'crypto'
import assert from 'assert'

describe(`database tests`, function () {
  const db = HoprDB.createMock()

  after(async function () {
    await db.close()
  })

  it('hasPacket', async function () {
    const packetTag = randomBytes(5)

    const present = await db.hasPacket(packetTag)

    assert(present == false, `Packet tag must not be present`)

    const secondTry = await db.hasPacket(packetTag)

    assert(secondTry == true, `Packet tag must be set`)
  })
})
