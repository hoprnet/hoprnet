import { type PeerDiscovery, type PeerDiscoveryEvents, symbol } from '@libp2p/interface-peer-discovery'
import type { PeerInfo } from '@libp2p/interface-peer-info'
import { EventEmitter, CustomEvent } from '@libp2p/interfaces/events'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'

// @ts-ignore libp2p interfaces type clash
class Discovery extends EventEmitter<PeerDiscoveryEvents> implements PeerDiscovery {
  private _running: boolean

  get [symbol](): true {
    return true
  }

  get [Symbol.toStringTag]() {
    return 'HoprConnect'
  }
  /**
   * Used by the isPeerDiscovery function
   */
  public symbol = true

  constructor() {
    super()

    this._running = false
  }

  public get running(): boolean {
    return this._running
  }

  public async start(): Promise<void> {
    this._running = true
  }

  public async stop(): Promise<void> {
    this._running = false
  }

  public _peerDiscovered(id: PeerId, multiaddrs: Multiaddr[]): void {
    if (!this._running) {
      return
    }

    this.dispatchEvent(new CustomEvent<PeerInfo>('peer', { detail: { id, multiaddrs, protocols: [] } }))
  }
}

export { Discovery }
