export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

export type InterfaceWithoutPort = Omit<Interface, 'port'>
