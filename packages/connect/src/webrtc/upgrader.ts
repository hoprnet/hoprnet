import SimplePeer from 'simple-peer'
import debug from 'debug'

import { Multiaddr } from 'multiaddr'
import type { PeerStoreType, HoprConnectOptions } from '../types'
import { CODE_IP4, CODE_TCP, CODE_UDP } from '../constants'
import type PeerId from 'peer-id'
import { AbortError } from 'abortable-iterator'

const wrtc = require('wrtc')

const DEBUG_PREFIX = `hopr-connect:webrtc`

const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

// @TODO adjust this
export const MAX_STUN_SERVERS = 23

/**
 * Check if we can use given Multiaddr as STUN server
 * @param ma Multiaddr to check
 * @returns true if given Multiaddr can be used as STUN server
 */
function isUsableMultiaddr(ma: Multiaddr): boolean {
  const tuples = ma.tuples()

  return tuples[0].length >= 2 && tuples[0][0] == CODE_IP4 && [CODE_UDP, CODE_TCP].includes(tuples[1][0])
}

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
function publicNodesToRTCServers(peerData: PeerStoreType[]): RTCIceServer[] {
  return Array.from({ length: peerData.length }, (_, index: number) => {
    const entry = peerData[index]

    return {
      urls:
        entry.multiaddrs.length == 1
          ? multiaddrToIceServer(entry.multiaddrs[0])
          : entry.multiaddrs.map(multiaddrToIceServer)
    }
  })
}

/**
 * Encapsulate configuration used to create WebRTC instances
 */
class WebRTCUpgrader {
  public rtcConfig?: RTCConfiguration
  private publicNodes: PeerStoreType[]

  private _onNewPublicNode: WebRTCUpgrader['onNewPublicNode'] | undefined
  private _onOfflineNode: WebRTCUpgrader['onOfflineNode'] | undefined

  constructor(private options: HoprConnectOptions) {
    this.publicNodes = []
  }

  /**
   * Attach event listeners to handle newly discovered public nodes and offline public nodes
   */
  public start(): void {
    this._onNewPublicNode = this.onNewPublicNode.bind(this)
    this._onOfflineNode = this.onOfflineNode.bind(this)

    this.options.initialNodes?.forEach(this._onNewPublicNode)

    if (this.options.publicNodes != undefined) {
      this.options.publicNodes.on('addPublicNode', this._onNewPublicNode)
      this.options.publicNodes.on('removePublicNode', this._onOfflineNode)
    }
  }

  /**
   * Unassign event listeners
   */
  public stop(): void {
    if (
      this.options.publicNodes != undefined &&
      this._onNewPublicNode != undefined &&
      this._onOfflineNode != undefined
    ) {
      this.options.publicNodes.removeListener('addPublicNode', this._onNewPublicNode)
      this.options.publicNodes.removeListener('removePublicNode', this._onOfflineNode)
    }
  }

  /**
   * Called on newly discovered public nodes
   * @param peer PeerId and its Multiaddrs
   * @returns
   */
  private onNewPublicNode(peer: PeerStoreType): void {
    if (
      this.rtcConfig != undefined &&
      this.rtcConfig.iceServers != undefined &&
      this.rtcConfig.iceServers.length >= MAX_STUN_SERVERS
    ) {
      return
    }

    let entryIndex = this.publicNodes.findIndex((entry: PeerStoreType) => entry.id.equals(peer.id))

    if (entryIndex < 0) {
      const usableAddresses = peer.multiaddrs.filter(isUsableMultiaddr)

      if (usableAddresses.length > 0) {
        this.publicNodes.unshift({ id: peer.id, multiaddrs: usableAddresses })
      }

      this.updateRTCConfig()
      return
    }

    let addrsChanged = false

    for (const ma of peer.multiaddrs) {
      // Also try "TCP addresses" as we expect that node is listening on TCP *and* UDP
      if (!isUsableMultiaddr(ma)) {
        verbose(`Dropping potential STUN ${ma.toString()} because format is invalid`)
        continue
      }

      if (this.publicNodes[entryIndex].multiaddrs.some(ma.equals.bind(ma))) {
        continue
      }

      addrsChanged = true
      this.publicNodes[entryIndex].multiaddrs.unshift(ma)
    }

    if (!addrsChanged) {
      return
    }

    this.updateRTCConfig()
  }

  /**
   * Called whenever a peer is considered offline
   * @param peer peer who is considered offline
   */
  private onOfflineNode(peer: PeerId): void {
    if (this.rtcConfig == undefined || this.rtcConfig.iceServers == undefined) {
      return
    }

    this.publicNodes = this.publicNodes.filter((entry: PeerStoreType) => !entry.id.equals(peer))

    this.updateRTCConfig()
  }

  private updateRTCConfig(): void {
    this.rtcConfig = {
      ...this.rtcConfig,
      iceServers: publicNodesToRTCServers(this.publicNodes)
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
      config: this.rtcConfig
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
