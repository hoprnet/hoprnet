declare module 'libp2p-utils/src/ip-port-to-multiaddr' {
  type Multiaddr = import('multiaddr')

  export default function toMultiaddr(ip: string, port: number): Multiaddr
}
