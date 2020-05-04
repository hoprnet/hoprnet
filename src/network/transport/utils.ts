import Multiaddr from 'multiaddr'
import os from 'os'
import { resolve } from 'path'

const ProtoFamily = { ip4: 'IPv4', ip6: 'IPv6' }

type AddressFamily = 'IPv4' | 'IPv6'
type MultiaddrFamily = 'ip4' | 'ip6'

export function multiaddrToNetConfig(addr: Multiaddr) {
  const listenPath = addr.getPath()
  // unix socket listening
  if (listenPath) {
    return resolve(listenPath)
  }
  // tcp listening
  return addr.toOptions()
}

export function getMultiaddrs(proto: MultiaddrFamily, ip: string, port: number) {
  const toMa = (ip: string) => Multiaddr(`/${proto}/${ip}/tcp/${port}`)
  return (isAnyAddr(ip) ? getNetworkAddrs(ProtoFamily[proto] as AddressFamily) : [ip]).map(toMa)
}

export function isAnyAddr(ip: string): boolean {
  return ['0.0.0.0', '::'].includes(ip)
}

function getNetworkAddrs(family: AddressFamily) {
  let interfaces = os.networkInterfaces()
  let addresses: string[] = []

  for (let iface of Object.values(interfaces)) {
    for (let netAddr of iface) {
      if (netAddr.family === family) {
        addresses.push(netAddr.address)
      }
    }
  }

  return addresses
}
