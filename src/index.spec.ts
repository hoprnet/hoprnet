/**
 * DO NOT DELETE THIS FILE
 *
 * The purpose of this file is to check whether the specified types can be used as intended.
 */

import HoprCoreConnector from '.'

const coreConnector = new HoprCoreConnector()

coreConnector.constants.CHAIN_NAME

coreConnector.types.AccountId.SIZE

coreConnector.start()

coreConnector.utils.hash(new Uint8Array(123).fill(0x00))

async function main() {
  const channel = await coreConnector.channel.create(coreConnector, new Uint8Array(), () => Promise.resolve(new Uint8Array()))

  const ticket = await channel.ticket.create(channel, new coreConnector.types.Balance(1), new coreConnector.types.Hash())

  ticket.signature.recovery * 2

  ticket.signature.signature.length
}

main()