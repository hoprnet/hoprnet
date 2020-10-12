import type { Handler } from '../@types/transport'

interface AbstractInteraction {
  protocols: string[]
  handler(struct: Handler): void
  interact(...props: any[]): any
}

export { AbstractInteraction }
