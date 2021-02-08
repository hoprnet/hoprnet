import type { Connection, MuxedStream } from 'libp2p'

interface AbstractInteraction {
  protocols: string[]
  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }): void
  interact(...props: any[]): any
}

export { AbstractInteraction }
