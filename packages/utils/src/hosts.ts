export interface NetOptions {
  ip: string
  port: number
}

export type Hosts = {
  ip4?: NetOptions
  ip6?: NetOptions
}

export function parseHosts(): Hosts {
  const hosts: Hosts = {}

  if (process.env['HOST_IPV4'] !== undefined) {
    const str = process.env['HOST_IPV4'].replace(/\/\/.+/, '').trim()
    const params = str.match(/([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\:([0-9]{1,6})/)
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv4 host. Got <${str}>`)
    }

    hosts.ip4 = {
      ip: params[1],
      port: parseInt(params[2])
    }
  }

  if (process.env['HOST_IPV6'] !== undefined) {
    const str = process.env['HOST_IPV6'].replace(/\/\/.+/, '').trim()
    const params = str.match(
      /\[([0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|\:\:[0-9a-fA-F]{1,4}|\:\:)\]\:([0-9]{1,6})/
    )
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv6 host. Got <${str}>`)
    }

    hosts.ip6 = {
      ip: params[1],
      port: parseInt(params[2])
    }
  }

  return hosts
}
