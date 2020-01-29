import pull from 'pull-stream'
import lp from 'pull-length-prefixed'
import Hopr from '../../'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { PROTOCOL_ONCHAIN_KEY } from '../../constants'
import { AbstractInteraction } from '../abstractInteraction'
import PeerId from 'peer-id'

class OnChainKey<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, PROTOCOL_ONCHAIN_KEY)
  }

  handle(protocol: any, conn: pull.Sink<Buffer>) {
    pull(pull.once(this.node.paymentChannels.self.keyPair.publicKey), lp.encode(), conn)
  }

  interact(counterparty: PeerId): Promise<Uint8Array> {
    return new Promise<Uint8Array>((resolve, reject) => {
      let resolved = false
      this.node.dialProtocol(counterparty, PROTOCOL_ONCHAIN_KEY, (err: Error | null, conn: pull.Source<Buffer>) => {
        pull(
          conn,
          lp.decode(),
          pull.drain((data?: Buffer) => {
            if (data == null || data.length == 0) {
              return reject(Error(`received ${data} but expected a public key`))
            }

            return resolve(data)
          })
        )
      })
    })
  }
}

export { OnChainKey }
