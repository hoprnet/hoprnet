import type { PeerId } from '@libp2p/interface-peer-id'

import { randomInteger, debug, retimer, pickVersion } from '@hoprnet/hopr-utils'
import type { Components } from '@libp2p/interfaces/components'

import type { SendMessage } from '../index.js'
import { pipe } from 'it-pipe'
import { reply_to_ping, HeartbeatConfig, Network, Pinger, PeerOrigin } from '../../lib/core_network.js'
import { core_network_set_panic_hook } from '../../lib/core_network.js'
core_network_set_panic_hook()

const log = debug('hopr-core:heartbeat')
const error = debug('hopr-core:heartbeat:error')

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../package.json' assert { type: 'json' }
import { peerIdFromString } from '@libp2p/peer-id'

const NORMALIZED_VERSION = pickVersion(pkg.version)

export default class Heartbeat {
  private stopHeartbeatInterval: (() => void) | undefined
  private protocolHeartbeat: string | string[]

  protected config: HeartbeatConfig // protected for testing
  protected pinger: Pinger // protected for testing

  constructor(
    protected networkPeers: Network, // protected for testing
    private libp2pComponents: Components,
    protected sendMessage: SendMessage,
    environmentId: string,
    config: HeartbeatConfig
  ) {
    this.config = config

    this.protocolHeartbeat = [
      // current
      `/hopr/${environmentId}/heartbeat/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${environmentId}/heartbeat`
    ]

    this.pinger = Pinger.build(
      environmentId,
      NORMALIZED_VERSION,
      (peer: string, result: number | undefined) => this.networkPeers.refresh(peer, result),
      (msg: Uint8Array, dest: string): Promise<Uint8Array[]> =>
        this.sendMessage(peerIdFromString(dest), this.protocolHeartbeat, msg, true)
    )

    this.pingNode = this.pingNode.bind(this)
  }

  private errHandler(err: any) {
    error(`Error while processing heartbeat request`, err)
  }

  public async start() {
    this.libp2pComponents.getRegistrar().handle(this.protocolHeartbeat, async ({ connection, stream }) => {
      let remote = connection.remotePeer.toString()
      if (this.networkPeers.contains(remote)) {
        this.networkPeers.refresh(remote, Date.now())
      } else {
        this.networkPeers.register(remote, PeerOrigin.IncomingConnection)
      }

      try {
        await pipe(
          stream.source,
          async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
            yield* {
              [Symbol.asyncIterator]() {
                return reply_to_ping(source[Symbol.asyncIterator]())
              }
            }
          },
          stream.sink
        )
      } catch (err) {
        this.errHandler(err)
      }
    })

    this.startHeartbeatInterval()
    log(`Heartbeat started`)
  }

  public stop() {
    this.stopHeartbeatInterval?.()
    log(`Heartbeat stopped`)
  }

  /**
   * Attempts to ping another peer.
   * @param destination id of the node to ping
   * @returns a Promise of a pingResult object with property `lastSeen < 0` if there were a timeout
   */
  public async pingNode(destination: PeerId) {
    log(`ping ${destination.toString()}`)

    await this.pinger.ping([destination.toString()])
  }

  /**
   * Starts the periodic check
   */
  private startHeartbeatInterval() {
    const periodicCheck = async function (this: Heartbeat) {
      try {
        const thresholdTime = Date.now() - Number(this.config.heartbeat_threshold)
        log(`Checking nodes since ${thresholdTime} (${new Date(thresholdTime).toLocaleString()})`)

        await this.pinger.ping(this.networkPeers.peers_to_ping(BigInt(thresholdTime)))
      } catch (err) {
        log('FATAL ERROR IN HEARTBEAT', err)
      }
    }.bind(this)

    this.stopHeartbeatInterval = retimer(
      periodicCheck,
      // Prevent nodes from querying each other at the very same time
      () =>
        randomInteger(this.config.heartbeat_interval, this.config.heartbeat_interval + this.config.heartbeat_variance)
    )
  }
}
