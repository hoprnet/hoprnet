import SimplePeer from 'simple-peer'

import { Multiaddr } from '@multiformats/multiaddr'
import type { PeerStoreType } from '../types.js'
import type { ConnectComponents, ConnectInitializable } from '../components.js'

import { AbortError } from 'abortable-iterator'

import errCode from 'err-code'

// No types for wrtc
// @ts-ignore
import wrtc from 'wrtc'

// @TODO adjust this
export const MAX_STUN_SERVERS = 23

/**
 * Converts a Multiaddr into an ICEServer string
 * @param ma Multiaddr to convert
 * @returns ICEServer representation of the given Multiaddr
 */
export function multiaddrToIceServer(ma: Multiaddr): string {
  const options = ma.toOptions()

  return `stun:${options.host}:${options.port}`
}

/**
 * Converts PeerData consisting of a PeerId and their Multiaddrs into a configuration
 * object to be used with RTCPeerConnection
 * @param peerData PeerIds and their Multiaddrs
 * @returns a configuration object to be used with RTCPeerConnection
 */
function publicNodesToRTCServers(peerData: IterableIterator<PeerStoreType>): RTCIceServer[] {
  const result: RTCIceServer[] = []
  for (const peer of peerData) {
    result.push({
      urls:
        peer.multiaddrs.length == 1
          ? multiaddrToIceServer(peer.multiaddrs[0])
          : peer.multiaddrs.map(multiaddrToIceServer)
    })
  }

  return result
}

/**
 * Encapsulate configuration used to create WebRTC instances
 */
class WebRTCUpgrader implements ConnectInitializable {
  private iceServers: RTCIceServer[]
  private lastEntryNodeUpdate: number | undefined

  private connectComponents: ConnectComponents | undefined

  constructor() {
    this.iceServers = []
  }

  public initConnect(connectComponents: ConnectComponents) {
    this.connectComponents = connectComponents
  }

  public getConnectComponents(): ConnectComponents {
    if (this.connectComponents == null) {
      throw errCode(new Error('connectComponents not set'), 'ERR_SERVICE_MISSING')
    }

    return this.connectComponents
  }

  private getRTCConfig(): RTCConfiguration {
    if (
      this.lastEntryNodeUpdate == undefined ||
      this.lastEntryNodeUpdate < this.getConnectComponents().getEntryNodes().lastUpdate
    ) {
      this.iceServers = publicNodesToRTCServers(
        function* (this: WebRTCUpgrader) {
          yield* this.getConnectComponents().getEntryNodes().getAvailableEntryNodes()
          yield* this.getConnectComponents().getEntryNodes().getUncheckedEntryNodes()
        }.call(this)
      )
    }
    return {
      iceServers: this.iceServers
    }
  }

  /**
   * Creates an outbound WebRTC instance
   * @param _signal @TODO implement this
   * @returns the WebRTC instance
   */
  public upgradeOutbound(_signal?: AbortSignal) {
    return this.connect(true)
  }

  /**
   * Creates an inbound WebRTC instance
   * @param _signal @TODO implement this
   * @returns the WebRTC instance
   */
  public upgradeInbound(_signal?: AbortSignal) {
    return this.connect(false)
  }

  /**
   * Creates a configured WebRTC instance and attaches basic
   * event listeners
   * @param initiator true if initiator
   * @param signal abort signal
   * @returns a configured WebRTC instance
   */
  private connect(initiator: boolean, signal?: AbortSignal) {
    const channel = new SimplePeer({
      wrtc,
      initiator,
      trickle: true,
      allowHalfTrickle: true,
      config: this.getRTCConfig()
    })

    if (signal) {
      const onAbort = () => {
        done(new AbortError())
      }

      let finished = false

      const done = (err?: Error) => {
        if (finished) {
          return
        }
        finished = true

        channel.removeListener('close', done)
        channel.removeListener('error', done)
        channel.removeListener('connect', done)

        signal?.removeEventListener('abort', onAbort)

        if (err) {
          channel.destroy()
        }
      }

      // Unassign abort handler once connection attempt failed,
      // connection got closed or connection succeeded
      channel.on('close', done)
      channel.on('error', done)
      channel.on('connect', done)

      signal?.addEventListener('abort', onAbort)
    }

    return channel
  }
}

export { WebRTCUpgrader }
