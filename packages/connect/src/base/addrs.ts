import { networkInterfaces, type NetworkInterfaceInfo } from 'os'
import { isLocalhost, ipToU8aAddress, isPrivateAddress, isLinkLocaleAddress } from '@hoprnet/hopr-utils'

import Debug from 'debug'
const log = Debug('hopr-connect')

type AddrOptions = {
  interface?: string
  useIPv4?: boolean
  useIPv6?: boolean
  includePrivateIPv4?: boolean
  includeLocalhostIPv4?: boolean
  includeLocalhostIPv6?: boolean
}

type AddressType = {
  address: string
  family: 'IPv4' | 'IPv6'
  port: number
}

function validateOptions(opts: AddrOptions) {
  if (opts == undefined) {
    return
  }

  if (opts.useIPv4 != true && opts.useIPv6 != true) {
    throw Error(`Must use either IPv4 or IPv6 but cannot use none.`)
  }

  if (opts.useIPv4 == false && (opts.includePrivateIPv4 || opts.includeLocalhostIPv4)) {
    throw Error(`Contradiction in opts. Cannot add private or local IPv4 address if IPv4 is disabled.`)
  }

  if (opts.useIPv6 == false && opts.includeLocalhostIPv6) {
    throw Error(`Contradiction in opts. Cannot add private or local IPv6 address if IPv6 is disabled.`)
  }
}

function getAddrsOfInterface(iface: string) {
  let ifaceAddrs = networkInterfaces()[iface]

  if (ifaceAddrs == undefined) {
    log(
      `Interface <${iface}> does not exist on this machine. Available are <${Object.keys(networkInterfaces()).join(
        ', '
      )}>`
    )
    return []
  }

  return ifaceAddrs
}

export function getAddrs(
  port: number,
  options: AddrOptions,
  __fakeInterfaces?: ReturnType<typeof networkInterfaces>
): AddressType[] {
  validateOptions(options)

  let interfaces: (NetworkInterfaceInfo[] | undefined)[]

  if (options?.interface) {
    interfaces = [getAddrsOfInterface(options.interface)]
  } else {
    interfaces = Object.values(__fakeInterfaces ?? networkInterfaces())
  }

  const multiaddrs: AddressType[] = []

  for (const iface of interfaces) {
    if (iface == undefined) {
      continue
    }

    for (const address of iface) {
      const u8aAddr = ipToU8aAddress(address.address, address.family)

      if (isLinkLocaleAddress(u8aAddr, address.family)) {
        continue
      }

      if (isPrivateAddress(u8aAddr, address.family)) {
        if (address.family === 'IPv4' && options.includePrivateIPv4 != true) {
          continue
        }
      }

      if (isLocalhost(u8aAddr, address.family)) {
        if (address.family === 'IPv4' && options.includeLocalhostIPv4 != true) {
          continue
        }
        if (address.family === 'IPv6' && options.includeLocalhostIPv6 != true) {
          continue
        }
      }

      if (address.family === 'IPv4' && options.useIPv4 != true) {
        continue
      }

      if (address.family === 'IPv6' && options.useIPv6 != true) {
        continue
      }

      multiaddrs.push({ ...address, port })
    }
  }

  return multiaddrs
}
