import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import { randomBytes, createHash } from 'crypto'

import type { AbstractInteraction } from '../abstractInteraction'

import pipe from 'it-pipe'
import chalk from 'chalk'

import { PROTOCOL_HEARTBEAT } from '../../constants'

import PeerInfo from 'peer-info'
import type PeerId from 'peer-id'

import { u8aEquals } from '../../utils'

const HASH_FUNCTION = 'blake2s256'

class Heartbeat<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_HEARTBEAT]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: any, stream: any }) {
    let events = this.node.network.heartbeat
    pipe(
      struct.stream,
      (source: any) => {
        return (async function * () {
          for await (const msg of source) {
            events.emit('beat', struct.connection.remotePeer)
            yield createHash(HASH_FUNCTION).update(msg.slice()).digest()
          }
        })()
      },
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo | PeerId, timeout: number): Promise<Uint8Array> {
    let struct: {
      stream: any
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err: Error) => {
        return this.node.peerRouting.findPeer(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty).then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      this.node.log(`Could not query ${(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty).toB58String()} for other nodes. Error was: ${chalk.red(err.message)}.`)
      return
    }

    const challenge = randomBytes(16)
    const expectedResponse = createHash(HASH_FUNCTION).update(challenge).digest()

    await pipe(
      /** prettier-ignore */
      [challenge],
      struct.stream,
      async (source: AsyncIterable<Buffer>) => {
        let done = false
        for await (const msg of source) {
          if (done == true) {
            continue
          }

          if (u8aEquals(msg, expectedResponse)) {
            done = true
          }
        }
      }
    )
  }
}

export { Heartbeat }
