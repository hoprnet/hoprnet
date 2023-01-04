// Use errCode library to add metadata to errors and align with libp2p transport interface
import errCode from 'err-code'

import type { WebRTCUpgrader } from './webrtc/upgrader.js'
import type { Filter } from './filter.js'
import type { EntryNodes } from './base/entry.js'
import type { Relay } from './relay/index.js'

import { isInitializable, type Initializable, type Components } from '@libp2p/interfaces/components'
import { type Startable, isStartable } from '@libp2p/interfaces/startable'

export interface ConnectInitializable {
  initConnect: (components: ConnectComponents) => void
}

export function isConnectInitializable(obj: any): obj is ConnectInitializable {
  return obj != null && typeof obj.initConnect === 'function'
}

export interface ConnectComponentsInit {
  addressFilter?: Filter
  entryNodes?: EntryNodes
  relay?: Relay
  webRTCUpgrader?: WebRTCUpgrader
}

export class ConnectComponents implements Startable, Initializable {
  private addressFilter: Filter | undefined
  private entryNodes: EntryNodes | undefined
  private relay: Relay | undefined
  private webRTCUpgrader: WebRTCUpgrader | undefined

  private _isStarted: boolean

  // Pass libp2p internals
  public init(components: Components) {
    for (const module of Object.values(this)) {
      if (isInitializable(module)) {
        module?.init(components)
      }
    }
  }

  constructor(init: ConnectComponentsInit) {
    this._isStarted = false

    if (init.addressFilter != null) {
      this.setAddressFilter(init.addressFilter)
    }

    if (init.entryNodes != null) {
      this.setEntryNodes(init.entryNodes)
    }

    if (init.relay != null) {
      this.setRelay(init.relay)
    }

    if (init.webRTCUpgrader != null) {
      this.setWebRTCUpgrader(init.webRTCUpgrader)
    }
  }

  public isStarted() {
    return this._isStarted
  }

  async beforeStart(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.beforeStart?.())
      }
    }

    await Promise.all(promises)
  }

  async start(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.start())
      }
    }

    await Promise.all(promises)

    this._isStarted = true
  }

  async afterStart(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.afterStart?.())
      }
    }

    await Promise.all(promises)
  }

  async beforeStop(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.beforeStop?.())
      }
    }

    await Promise.all(promises)
  }

  async stop(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.stop())
      }
    }

    await Promise.all(promises)

    this._isStarted = false
  }

  async afterStop(): Promise<void> {
    const promises: (Promise<void> | void)[] = []

    for (const module of Object.values(this)) {
      if (isStartable(module)) {
        promises.push(module.afterStop?.())
      }
    }

    await Promise.all(promises)
  }

  setAddressFilter(addressFilter: Filter) {
    if (isConnectInitializable(addressFilter)) {
      addressFilter.initConnect(this)
    }

    this.addressFilter = addressFilter
  }

  getAddressFilter(): Filter {
    if (this.addressFilter == null) {
      throw errCode(new Error('addressManager not set'), 'ERR_SERVICE_MISSING')
    }

    return this.addressFilter
  }

  setEntryNodes(entryNodes: EntryNodes) {
    if (isConnectInitializable(entryNodes)) {
      entryNodes.initConnect(this)
    }

    this.entryNodes = entryNodes
  }

  getEntryNodes(): EntryNodes {
    if (this.entryNodes == null) {
      throw errCode(new Error('entryNodes not set'), 'ERR_SERVICE_MISSING')
    }

    return this.entryNodes
  }

  setRelay(relay: Relay) {
    if (isConnectInitializable(relay)) {
      relay.initConnect(this)
    }

    this.relay = relay
  }

  getRelay(): Relay {
    if (this.relay == null) {
      throw errCode(new Error('relay not set'), 'ERR_SERVICE_MISSING')
    }

    return this.relay
  }

  setWebRTCUpgrader(webRTCUpgrader: WebRTCUpgrader) {
    if (isConnectInitializable(webRTCUpgrader)) {
      webRTCUpgrader.initConnect(this)
    }

    this.webRTCUpgrader = webRTCUpgrader
  }

  getWebRTCUpgrader(): WebRTCUpgrader {
    if (this.webRTCUpgrader == null) {
      throw errCode(new Error('webRTCUpgrader not set'), 'ERR_SERVICE_MISSING')
    }

    return this.webRTCUpgrader
  }
}
