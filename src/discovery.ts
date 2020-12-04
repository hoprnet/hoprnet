import { PeerDiscovery } from 'libp2p-interfaces'
import { EventEmitter } from 'events'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'

class Discovery extends EventEmitter implements PeerDiscovery {
  private _running: boolean

  public tag = 'HoprConnect'

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
