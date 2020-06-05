/**
 * DO NOT DELETE THIS FILE
 *
 * The purpose of this file is to check whether the specified types can be used as intended.
 */

import HoprCoreConnector from '.'
import type { LevelUp } from 'levelup'
import type { Balance, Hash, AccountId } from './types'

async function main() {
  const coreConnector = await HoprCoreConnector.create((undefined as unknown) as LevelUp)

  coreConnector.constants.CHAIN_NAME

  coreConnector.types.AccountId.SIZE

  coreConnector.start()

  coreConnector.indexer?.has((undefined as unknown) as AccountId, (undefined as unknown) as AccountId)

  coreConnector.utils.hash(new Uint8Array(123).fill(0x00))

  const channel = await coreConnector.channel.create(new Uint8Array(), () => Promise.resolve(new Uint8Array()))

  const ticket = await channel.ticket.create((undefined as unknown) as Balance, (undefined as unknown) as Hash)

  ticket.signature.recovery * 2

  ticket.signature.signature.length
}

main()
