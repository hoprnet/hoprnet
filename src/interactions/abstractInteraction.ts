import Hopr from '..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import pull from 'pull-stream'

abstract class AbstractInteraction<Chain extends HoprCoreConnectorInstance> {
  constructor(node: Hopr<Chain>, protocol: string) {
    node.handle(protocol, this.handle)
  }

  abstract handle(protocol: any, conn: pull.Through<Buffer, Buffer>): void
}

export { AbstractInteraction }
