import {
  Address as TsAddress,
  Balance as TsBalance,
  ChannelEntry as TsChannelEntry,
  Hash as TsHash,
  PublicKey as TsPublicKey,
  Response as TsResponse,
  Signature as TsSignature,
  SIGNATURE_LENGTH,
  stringToU8a,
  Ticket as TsTicket,
  UINT256
} from '@hoprnet/hopr-utils'

import { Balance, BalanceType, ChannelEntry, ChannelStatus, Response, Ticket, U256 } from '../lib/core_types.js'
import assert from 'assert'

import { Hash, PublicKey, Signature } from './cryptography.js'
import { randomBytes } from 'crypto'
import BN from 'bn.js'

let private_key_1 = stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775')
let pub_key_1 = PublicKey.from_privkey(private_key_1)

let private_key_2 = stringToU8a('0x4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b')
let pub_key_2 = PublicKey.from_privkey(private_key_2)

describe('Rust - TS serialization/deserialization tests', async function () {
  it('ticket serialize/deserialize', async function() {

    let ts_ticket: TsTicket
    {
      const userA = TsAddress.deserialize(pub_key_1.to_address().serialize())
      const challenge = new TsResponse(Uint8Array.from(randomBytes(32))).toChallenge().toEthereumChallenge()
      const epoch = UINT256.fromString('1')
      const index = UINT256.fromString('1')
      const amount = new TsBalance(new BN(1))
      const winProb = UINT256.fromInverseProbability(new BN(1))
      const channelEpoch = UINT256.fromString('1')
      const signature = new TsSignature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
      ts_ticket = new TsTicket(userA, challenge, epoch, index, amount, winProb, channelEpoch, signature)
    }

    let rs_ticket: Ticket
    {
      const userA = pub_key_1.to_address()
      const challenge = new Response(Uint8Array.from(randomBytes(32))).to_challenge().to_ethereum_challenge()
      const epoch = U256.one()
      const index = U256.one()
      const amount = new Balance('1', BalanceType.HOPR)
      const winProb = U256.from_inverse_probability(U256.one())
      const channelEpoch = U256.one()
      const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
      rs_ticket = new Ticket(userA, challenge, epoch, index, amount, winProb, channelEpoch, signature)
    }

    assert(Ticket.deserialize(ts_ticket.serialize()).eq(rs_ticket))
    assert(TsTicket.deserialize(rs_ticket.serialize()) == ts_ticket)
  })

  it('channel entry serialize/deserialize', async function() {
    let rs_channel_entry = new ChannelEntry(
      pub_key_1,
      pub_key_2,
      new Balance('1', BalanceType.HOPR),
      Hash.create([stringToU8a('0xdeadbeef')]),
      U256.one(),
      U256.zero(),
      ChannelStatus.Closed,
      U256.one(),
      U256.one()
    )

    let ts_channel_entry = new TsChannelEntry(
      TsPublicKey.deserialize(pub_key_1.serialize(false)),
      TsPublicKey.deserialize(pub_key_2.serialize(false)),
      new TsBalance(new BN('1')),
      TsHash.create(stringToU8a('0xdeadbeef')),
      UINT256.fromString('1'),
      UINT256.fromString('0'),
      ChannelStatus.Open,
      UINT256.fromString('1'),
      UINT256.fromString('1'),
    )

    assert(ChannelEntry.deserialize(ts_channel_entry.serialize()).eq(rs_channel_entry))
    assert(TsChannelEntry.deserialize(rs_channel_entry.serialize()) == ts_channel_entry)
  })



})