import type Hopr from '..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import type { Handler } from '../network/transport/types'

interface AbstractInteraction<Chain extends HoprCoreConnector> {
  protocols: string[]
  node: Hopr<Chain>

  handler(struct: Handler): void

  interact(...props: any[]): any
}

export { AbstractInteraction }
