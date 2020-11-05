import { WebRTCConnection } from './webRTCConnection'
import { RelayConnection } from './relayConnection'
import PeerId from 'peer-id'
import type { MultiaddrConnection } from 'libp2p'
import Peer from 'simple-peer'

// @ts-ignore
import wrtc = require('wrtc')

import Pair from 'it-pair'

async function main() {
  const AliceBob = Pair()
  const BobAlice = Pair()

  const Alice = await PeerId.create({ keyType: 'secp256k1' })
  const Bob = await PeerId.create({ keyType: 'secp256k1' })

  const PeerAlice = new Peer({ wrtc, initiator: true, trickle: true })
  const PeerBob = new Peer({ wrtc, trickle: true })

  // await new Promise(resolve => {
  // })

  const a = new WebRTCConnection({
    conn: new RelayConnection({
      stream: {
        sink: AliceBob.sink,
        source: BobAlice.source
      } as MultiaddrConnection,
      counterparty: Bob,
      self: Alice,
      webRTC: PeerAlice,
      onReconnect: async () => {}
    }),
    self: Alice,
    counterparty: Bob,
    channel: PeerAlice
  })

  const b = new WebRTCConnection({
    conn: new RelayConnection({
      stream: {
        sink: BobAlice.sink,
        source: AliceBob.source
      } as MultiaddrConnection,
      self: Bob,
      counterparty: Alice,
      webRTC: PeerBob,
      onReconnect: async () => {}
    }),
    self: Bob,
    counterparty: Alice,
    channel: PeerBob
  })

  a.sink(
    (async function* () {
      while (true) {
        await new Promise((resolve) => setTimeout(resolve, 1000))
        yield new TextEncoder().encode(`fancy WebRTC message`)
      }
    })()
  )

  // b.sink(
  //   (async function* () {
  //     while (true) {
  //       await new Promise((resolve) => setTimeout(resolve, 1000))
  //       yield new TextEncoder().encode(`fancy WebRTC message`)
  //     }
  //   })()
  // )

  function foo({ done, value }: { done?: boolean | void; value?: Uint8Array | void }) {
    if (value) {
      console.log(new TextDecoder().decode(value))
    }

    if (!done) {
      b.source.next().then(foo)
    }
  }

  // function bar({ done, value }: { done?: boolean | void; value?: Uint8Array | void }) {
  //   if (value) {
  //     console.log(new TextDecoder().decode(value))
  //   }

  //   if (!done) {
  //     b.source.next().then(foo)
  //   }
  // }

  b.source.next().then(foo)
  //a.source.next().then(bar)

  //PeerBob.on('signal', (msg: any) => setTimeout(() => PeerAlice.signal(msg), 150))

  //PeerAlice.on('signal', (msg: any) => setTimeout(() => PeerBob.signal(msg), 150))
}

main()
