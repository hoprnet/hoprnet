import type { Handler } from 'libp2p'

interface AbstractInteraction {
  protocols: string[]
  handler(struct: Handler): void
  interact(...props: any[]): any
}

export { AbstractInteraction }
