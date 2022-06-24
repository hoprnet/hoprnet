import { type PeerDiscovery, symbol } from '@libp2p/interface-peer-discovery'
import { EventEmitter } from 'events'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'

class Discovery extends EventEmitter implements PeerDiscovery {
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

    this.emit('peer', { id, multiaddrs })
  }
}

export { Discovery }
