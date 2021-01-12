import type Hopr from '../..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import assert from 'assert'

import { randomBytes } from 'crypto'

import { UnacknowledgedTicket } from '.'

import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'

import { Types, Utils } from '@hoprnet/hopr-core-ethereum'
import { privKeyToPeerId } from '../../utils'
import { toU8a, u8aConcat, u8aToNumber } from '@hoprnet/hopr-utils'

import LevelUp from 'levelup'
import Memdown from 'memdown'

import * as DbKeys from '../../dbKeys'

describe(`check serialization and deserialization of ticket objects`, function () {
  function getNode(): Hopr<HoprCoreConnector> {
    return ({
      db: LevelUp(Memdown()),
      _dbKeys: DbKeys,
      paymentChannels: ({
        utils: Utils,
        types: new Types()
      } as unknown) as HoprCoreConnector
    } as unknown) as Hopr<HoprCoreConnector>
  }

  it('should create a winning ticket', async function () {
    const node = getNode()

    const peerA = await privKeyToPeerId(NODE_SEEDS[0])
    const peerB = await privKeyToPeerId(NODE_SEEDS[1])

    // const accountA = await node.paymentChannels.utils.pubKeyToAccountId(peerA.pubKey.marshal())
    const accountB = await node.paymentChannels.utils.pubKeyToAccountId(peerB.pubKey.marshal())

    const secretA = randomBytes(32)
    const secretB = randomBytes(32)

    const response = await node.paymentChannels.utils.hash(u8aConcat(secretA, secretB))
    const challenge = await node.paymentChannels.utils.hash(response)

    const unAcknowledgedTicket = new UnacknowledgedTicket(node.paymentChannels)

    const signedTicket = await node.paymentChannels.types.SignedTicket.create({
      bytes: unAcknowledgedTicket.buffer,
      offset: unAcknowledgedTicket.signedTicketOffset
    })

    const ticket = node.paymentChannels.types.Ticket.create(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset
      },
      {
        amount: new node.paymentChannels.types.Balance(1),
        counterparty: accountB,
        challenge,
        epoch: new node.paymentChannels.types.TicketEpoch(0),
        winProb: new node.paymentChannels.types.Hash(new Uint8Array(32).fill(0xff)),
        channelIteration: new node.paymentChannels.types.TicketEpoch(0)
      }
    )

    await ticket.sign(peerA.privKey.marshal(), undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset
    })

    assert(await unAcknowledgedTicket.verifySignature(peerA), 'signature must be valid')

    await node.db.put(node._dbKeys.UnAcknowledgedTickets(challenge), Buffer.from(unAcknowledgedTicket))

    const fromDbUnacknowledgedTicket = (await node.db.get(node._dbKeys.UnAcknowledgedTickets(challenge))) as Uint8Array

    assert(
      await new UnacknowledgedTicket(node.paymentChannels, {
        bytes: fromDbUnacknowledgedTicket.buffer,
        offset: fromDbUnacknowledgedTicket.byteOffset
      }).verifySignature(peerA),
      'signature must be valid'
    )

    const acknowledgedDbEntry = node.paymentChannels.types.AcknowledgedTicket.create(node.paymentChannels, undefined, {
      signedTicket,
      response: await node.paymentChannels.utils.hash(u8aConcat(secretA, secretB)),
      preImage: randomBytes(27),
      redeemed: false
    })

    const FIRST_TICKET = 1
    await node.db.put(
      Buffer.from(node._dbKeys.AcknowledgedTicketCounter()),
      Buffer.from(toU8a(FIRST_TICKET, DbKeys.ACKNOWLEDGED_TICKET_INDEX_LENGTH))
    )

    const counter = (await node.db.get(Buffer.from(node._dbKeys.AcknowledgedTicketCounter()))) as Uint8Array

    assert(u8aToNumber(counter) == FIRST_TICKET)

    await node.db.put(Buffer.from(node._dbKeys.AcknowledgedTickets(counter)), Buffer.from(acknowledgedDbEntry))

    //const fromDbtmp = await node.db.get(Buffer.from(node.dbKeys.AcknowledgedTickets(counter)))
  })
})
