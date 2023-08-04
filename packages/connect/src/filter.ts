import type { Multiaddr } from '@multiformats/multiaddr'
import type { ValidAddress } from './utils/index.js'
import type { Initializable, Components } from '@libp2p/interfaces/components'

import {
  u8aEquals,
  checkNetworks,
  isDappnodePrivateNetwork,
  isPrivateAddress,
  isLinkLocaleAddress,
  isReservedAddress,
  u8aAddrToString,
  getPrivateAddresses,
  isLocalhost,
  u8aAddressToCIDR,
  type Network
} from '@hoprnet/hopr-utils'
import { AddressType, parseAddress, type DirectAddress, type CircuitAddress } from './utils/index.js'

import errCode from 'err-code'
import Debug from 'debug'
import type { HoprConnectOptions } from './types.js'

const log = Debug('hopr-connect:filter')

const INVALID_PORTS = [0]

export class Filter implements Initializable {
  private announcedAddrs?: ValidAddress[]
  private listeningFamilies?: (AddressType.IPv4 | AddressType.IPv6)[]

  protected myPrivateNetworks: Network[]

  private components: Components | undefined

  constructor(private opts: HoprConnectOptions) {
    this.myPrivateNetworks = getPrivateAddresses()
  }

  public init(components: Components): void {
    this.components = components
  }

  public getComponents(): Components {
    if (this.components == null) {
      throw errCode(new Error('components not set'), 'ERR_SERVICE_MISSING')
    }

    return this.components
  }

  /**
   * Used to check whether addresses have already been attached
   */
  public get addrsSet(): boolean {
    return this.announcedAddrs != undefined && this.listeningFamilies != undefined
  }

  /**
   * Used to attach addresses once libp2p is initialized and
   * sockets are bound to network interfaces
   * @param announcedAddrs Addresses that are announced to other nodes
   * @param listeningAddrs Addresses to which we are listening
   */
  public setAddrs(announcedAddrs: Multiaddr[], listeningAddrs: Multiaddr[]): void {
    this.announcedAddrs = []
    for (const announcedAddr of announcedAddrs) {
      const parsed = parseAddress(announcedAddr)

      if (!parsed.valid) {
        continue
      }

      this.announcedAddrs.push(parsed.address)
    }

    this.listeningFamilies = []
    for (const listenAddr of listeningAddrs) {
      const parsed = parseAddress(listenAddr)

      if (!parsed.valid) {
        continue
      }

      switch (parsed.address.type) {
        case AddressType.IPv4:
        case AddressType.IPv6:
          if (!this.listeningFamilies.includes(parsed.address.type)) {
            this.listeningFamilies.push(parsed.address.type)
          }
          break
        case AddressType.P2P:
          continue
      }
    }
  }

  public filter(ma: Multiaddr): boolean {
    if (this.addrsSet) {
      return this.filterDial(ma)
    } else {
      return this.filterListening(ma)
    }
  }

  /**
   * Check whether we can listen to the given Multiaddr
   * @param ma Multiaddr to check
   * @returns true if address is usable
   */
  private filterListening(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid) {
      return false
    }

    switch (parsed.address.type) {
      case AddressType.IPv4:
      case AddressType.IPv6:
        return true
      case AddressType.P2P:
        log(`Can only listen to IP addresses: Given addr: ${ma.toString()}`)
        return false
    }
  }

  /**
   * Check whether it makes sense to dial the given address
   * @param ma Multiaddress to check
   * @returns true if considered dialable
   */
  private filterDial(ma: Multiaddr): boolean {
    const parsed = parseAddress(ma)

    if (!parsed.valid) {
      return false
    }

    switch (parsed.address.type) {
      case AddressType.P2P:
        return this.filterCircuitDial(parsed.address, ma)
      case AddressType.IPv4:
      case AddressType.IPv6:
        return this.filterDirectDial(parsed.address, ma)
    }
  }

  /**
   * Filter dial attempts using ciruit addresses
   * @param address parsed circuit address
   * @param ma original Multiaddr for logging
   * @returns
   */
  private filterCircuitDial(address: CircuitAddress, ma: Multiaddr): boolean {
    if (u8aEquals(address.relayer, this.getComponents().getPeerId().publicKey)) {
      log(`Prevented dial using self as relay node. Used addr: ${ma.toString()}`)
      return false
    }

    return true
  }

  /**
   * Filter dial attempts using direct addresses
   * @param address parsed direct address
   * @param ma original Multiaddr for logging
   * @returns
   */
  private filterDirectDial(address: DirectAddress, ma: Multiaddr): boolean {
    if (
      (process.env.DAPPNODE ?? 'false').toLowerCase() === 'true' &&
      isDappnodePrivateNetwork(address.address, address.type)
    ) {
      // Never attempt to dial internal container addresses of Dappnode machines
      return false
    }

    if (!this.listeningFamilies!.includes(address.type)) {
      // Prevent dialing IPv6 addresses when only listening to IPv4 and vice versa
      log(`Tried to dial ${address.type} address but listening to ${this.listeningFamilies!.join(', ')}`)
      return false
    }

    if (INVALID_PORTS.includes(address.port)) {
      log(`Tried to dial invalid port ${address.port}`)
      return false
    }

    if (isLinkLocaleAddress(address.address, address.type) || isReservedAddress(address.address, address.type)) {
      // Prevent dialing any link-locale addresses or reserved addresses
      return false
    } else if (isLocalhost(address.address, address.type)) {
      // If localhost connections are explicitly allowed, do not dial them
      if (!this.opts.allowLocalConnections) {
        // Do not pollute logs by rejecting localhost connections attempts
        return false
      }

      // Allow to dial localhost only if the port is different from all of those we're listening on
      if (
        this.announcedAddrs!.some(
          (announced: ValidAddress) =>
            announced.type !== AddressType.P2P &&
            isLocalhost(announced.address, announced.type) &&
            announced.type === address.type &&
            announced.port == address.port
        )
      ) {
        // Do not log anything to prevent too much log pollution
        return false
      }
    } else if (isPrivateAddress(address.address, address.type)) {
      // If private address connections are explicitly allowed, do not dial them
      if (!this.opts.allowPrivateConnections) {
        // Do not pollute logs by rejecting private address connections attempts
        return false
      }

      // If different private network, there is most likely no chance to establish a connection
      if (!checkNetworks(this.myPrivateNetworks, address.address, address.type)) {
        log(
          `Prevented dialing private address ${u8aAddrToString(address.address, address.type)}:${
            address.port
          } because not in our network(s): ${this.myPrivateNetworks
            .map((network) => `${u8aAddressToCIDR(network.networkPrefix, network.subnet, network.family)}`)
            .join(', ')}`
        )
        return false
      }
    }

    return this.filterSameHostDial(address, ma)
  }

  /**
   * Filter dial attempts to other nodes on same host
   * @param address parsed direct address
   * @param ma original Multiaddr for logging
   * @returns
   */
  private filterSameHostDial(address: DirectAddress, ma: Multiaddr) {
    // Allow multiple nodes on same host - independent of address type
    for (const announcedAddr of this.announcedAddrs!) {
      switch (announcedAddr.type) {
        case AddressType.IPv4:
        case AddressType.IPv6:
          if (address.type === announcedAddr.type && u8aEquals(announcedAddr.address, address.address)) {
            // Always allow dials to own address whenever port is different
            // and block if port is identical
            if (address.port == announcedAddr.port) {
              log(
                `Prevented dialing ${ma.toString()} because self listening on ${u8aAddrToString(
                  announcedAddr.address,
                  announcedAddr.type
                )}:${announcedAddr.port}`
              )
              return false
            }
          }
      }
    }

    return true
  }
}
