/**
 * DO NOT DELETE THIS FILE
 *
 * The purpose of this file is to check whether the specified types can be used as intended.
 */

import HoprCoreConnector from '.'
import type { LevelUp } from 'levelup'
import type { Balance, Hash, SignedTicket, Ticket, Signature } from './types'

async function main() {
  const coreConnector = await HoprCoreConnector.create((undefined as unknown) as LevelUp)

  coreConnector.constants.CHAIN_NAME

  coreConnector.types.AccountId.SIZE

  coreConnector.start()

  coreConnector.tickets.get(coreConnector, new Uint8Array())
  coreConnector.tickets.store(coreConnector, new Uint8Array(), (undefined as unknown) as SignedTicket<Ticket, Signature>)

  coreConnector.indexer.has(undefined, undefined)

  coreConnector.utils.hash(new Uint8Array(123).fill(0x00))

  const channel = await coreConnector.channel.create(coreConnector, new Uint8Array(), () => Promise.resolve(new Uint8Array()))

  const ticket = await channel.ticket.create(channel, (undefined as unknown) as Balance, (undefined as unknown) as Hash)

  ticket.signature.recovery * 2

  ticket.signature.signature.length
}

main()
