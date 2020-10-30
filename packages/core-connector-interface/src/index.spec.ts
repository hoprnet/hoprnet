/**
 * DO NOT DELETE THIS FILE
 *
 * The purpose of this file is to check whether the specified types can be used as intended.
 */

import HoprCoreConnector from '.'
import type {LevelUp} from 'levelup'
import type {Balance, Hash, Public, AcknowledgedTicket} from './types'

async function main() {
  const coreConnector = await HoprCoreConnector.create(
    (undefined as unknown) as LevelUp,
    (undefined as unknown) as Uint8Array
  )

  coreConnector.constants.CHAIN_NAME

  coreConnector.types.AccountId.SIZE

  await coreConnector.start()

  await coreConnector.indexer?.has((undefined as unknown) as Public, (undefined as unknown) as Public)

  await coreConnector.indexer?.has((undefined as unknown) as Public, (undefined as unknown) as Public)

  await coreConnector.utils.hash(new Uint8Array(123).fill(0x00))

  const channel = await coreConnector.channel.create(new Uint8Array(), () =>
    Promise.resolve((undefined as unknown) as Public)
  )

  const ticket = await channel.ticket.create(
    (undefined as unknown) as Balance,
    (undefined as unknown) as Hash,
    (undefined as unknown) as number
  )

  ticket.signature.recovery * 2

  ticket.signature.signature.length
}

main()
