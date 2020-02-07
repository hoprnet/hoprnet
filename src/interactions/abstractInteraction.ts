import Hopr from '..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

export type Sink = (source: AsyncIterable<Uint8Array>) => void

export type Source = AsyncIterator<Uint8Array>

export type Duplex = {
  sink: Sink
  source: Source
}

interface AbstractInteraction<Chain extends HoprCoreConnectorInstance> {
  protocols: string[]
  node: Hopr<Chain>

  handler(struct: { stream: Duplex }): void

  interact(...props: any[]): any
}

export { AbstractInteraction }
