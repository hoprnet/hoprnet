import LevelUp from 'levelup'
import PeerId from 'peer-id'
import Memdown from 'memdown'
import { randomBytes } from 'crypto'
import { EventEmitter } from 'events'

import { subscribeToAcknowledgements, sendAcknowledgement } from './acknowledgement'
import { createFirstChallenge } from '@hoprnet/hopr-utils'

import { Challenge } from '../../messages/challenge'
import { Packet } from '../../messages/packet'
// import assert from 'assert'

const SECRET_LENGTH = 32

function createFakeChain() {
  const acknowledge = () => {}
  const getChannel = () => ({
    acknowledge
  })

  return { getChannel }
}

function createFakeSendReceive(self: PeerId, counterparty: PeerId) {
  const event = new EventEmitter()

  const send = (destination: PeerId, _protocol: any, msg: Uint8Array) =>
    event.emit('msg', msg, destination.equals(self) ? counterparty : self)

  const subscribe = (_protocol: any, foo: (msg: Uint8Array, sender: PeerId) => any) => {
    event.on('msg', (msg, sender) => foo(msg, sender))
  }

  return {
    send,
    subscribe
  }
}

describe('packet interaction', function () {
  let self: PeerId
  let counterparty: PeerId

  const db = LevelUp(Memdown())

  before(async function () {
    ;[self, counterparty] = await Promise.all(Array.from({ length: 2 }, (_) => PeerId.create({ keyType: 'secp256k1' })))
  })

  it('acknowledgement workflow', function () {
    const chain = createFakeChain()

    const libp2p = createFakeSendReceive(self, counterparty)

    const secrets = Array.from({ length: 2 }, (_) => randomBytes(SECRET_LENGTH))

    const { ackChallenge } = createFirstChallenge(secrets)

    const challenge = Challenge.create(ackChallenge, self)

    const fakePacket = new Packet(new Uint8Array(), challenge, undefined as any)

    fakePacket.ownKey = secrets[0]

    subscribeToAcknowledgements(libp2p.subscribe, db, chain as any, self, () => {})

    sendAcknowledgement(fakePacket, self, libp2p.send, counterparty)
  })
})
