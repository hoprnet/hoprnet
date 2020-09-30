import type { Handler } from '../network/transport/types'

interface AbstractInteraction {
  protocols: string[]
  handler(struct: Handler): void
  interact(...props: any[]): any
}

export { AbstractInteraction }
