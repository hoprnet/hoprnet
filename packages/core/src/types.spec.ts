import {
  Address as TsAddress,
  Balance as TsBalance,
  ChannelEntry as TsChannelEntry,
  Hash as TsHash,
  PublicKey as TsPublicKey,
  Response as TsResponse,
  HalfKey as TsHalfKey,
  //HalfKeyChallenge as TsHalfKeyChallenge,
  Signature as TsSignature,
  SIGNATURE_LENGTH,
  stringToU8a,
  Ticket as TsTicket,
  UnacknowledgedTicket as TsUnacknowledgedTicket,
  UINT256, u8aToHex, toEthSignedMessageHash, privKeyToPeerId
} from '@hoprnet/hopr-utils'

import {AcknowledgementChallenge as TsAcknowledgementChallenge} from './messages/acknowledgementChallenge.js'
import {Acknowledgement as TsAcknowledgement} from './messages/acknowledgement.js'

import {
  Acknowledgement,
  AcknowledgementChallenge,
  Address,
  Balance,
  BalanceType,
  ChannelEntry,
  ChannelStatus, HalfKey,
  //HalfKeyChallenge,
  Response,
  Ticket,
  U256,
  Hash,
  PublicKey,
  Signature, ethereum_signed_hash,
  UnacknowledgedTicket
} from '../lib/core_types.js'
import assert from 'assert'

import { randomBytes } from 'crypto'
import BN from 'bn.js'

let private_key_1 = stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775')
let pub_key_1 = PublicKey.from_privkey(private_key_1)

let private_key_2 = stringToU8a('0x4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b')
let pub_key_2 = PublicKey.from_privkey(private_key_2)

describe('Rust - TS serialization/deserialization tests', async function () {
  it('eth hash tests', async function() {
    let hash_1 = TsHash.create(stringToU8a('0xdeadbeef'))
    let ethSigned_1 = toEthSignedMessageHash(hash_1)

    let hash_2 = Hash.create([stringToU8a('0xdeadbeef')])
    let ethSigned_2 = ethereum_signed_hash(hash_2)

    assert.equal(ethSigned_1.toHex(), '0x' + ethSigned_2.to_hex())
  })


  it('ticket serialize/deserialize', async function() {

    let challenge = Uint8Array.from(randomBytes(32))

    const user_1 = TsAddress.deserialize(pub_key_1.to_address().serialize())
    const challenge_1 = new TsResponse(challenge).toChallenge().toEthereumChallenge()
    const epoch_1 = UINT256.fromString('1')
    const index_1 = UINT256.fromString('1')
    const amount_1 = new TsBalance(new BN(1))
    const winProb_1 = UINT256.fromInverseProbability(new BN(1))
    const channelEpoch_1 = UINT256.fromString('1')
    const signature_1 = new TsSignature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    let ts_ticket = new TsTicket(user_1, challenge_1, epoch_1, index_1, amount_1, winProb_1, channelEpoch_1, signature_1)

    const user_2 = pub_key_1.to_address() as Address
    const challenge_2 = new Response(challenge).to_challenge().to_ethereum_challenge()
    const epoch_2 = U256.one()
    const index_2 = U256.one()
    const amount_2 = new Balance('1', BalanceType.HOPR)
    const winProb_2 = U256.from_inverse_probability(U256.one())
    const channelEpoch_2 = U256.one()
    const signature_2 = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    let rs_ticket = new Ticket(user_2, challenge_2, epoch_2, index_2, amount_2, winProb_2, channelEpoch_2, signature_2)

    let a = u8aToHex(ts_ticket.serialize());
    let b = u8aToHex(rs_ticket.serialize());
    assert.equal(a,b)

    assert(Ticket.deserialize(ts_ticket.serialize()).eq(rs_ticket), "ticket serde test failed")
  })

  it('channel entry', async function() {
    let rs_channel_entry = new ChannelEntry(
      PublicKey.from_privkey(private_key_1),
      PublicKey.from_privkey(private_key_2),
      new Balance('1', BalanceType.HOPR),
      Hash.create([stringToU8a('0xdeadbeef')]),
      U256.one(),
      U256.zero(),
      ChannelStatus.Open,
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

    let a = u8aToHex(ts_channel_entry.serialize());
    let b = u8aToHex(rs_channel_entry.serialize());
    assert.equal(a,b)

    assert(ChannelEntry.deserialize(ts_channel_entry.serialize()).eq(rs_channel_entry), "channel entry serde test failed")
  })

  it('acknowledgment', async function() {

    let rs_hk = new HalfKey(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
    let rs_hkc = rs_hk.to_challenge()
    let rs_akc = new AcknowledgementChallenge(rs_hkc, private_key_1)
    let rs_ack = new Acknowledgement(new AcknowledgementChallenge(rs_hkc, private_key_1), rs_hk, private_key_2)

    let peer_id_1 = privKeyToPeerId(private_key_1)
    let peer_id_2 = privKeyToPeerId(private_key_2)

    let ts_hk = new TsHalfKey(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
    let ts_hkc = ts_hk.toChallenge()
    let ts_akc = TsAcknowledgementChallenge.create(ts_hkc, peer_id_1)
    let ts_ack = TsAcknowledgement.create(ts_akc, ts_hk, peer_id_2)

    let a = u8aToHex(rs_ack.serialize())
    let b = u8aToHex(ts_ack.serialize())
    assert.equal(a,b)
    let d_ack = Acknowledgement.deserialize(ts_ack.serialize())
    assert(d_ack.validate(pub_key_1, pub_key_2), "couldn't validate acks")
    assert(d_ack.eq(rs_ack), "ack serde test failed")

    let aa = u8aToHex(ts_akc.serialize())
    let bb = u8aToHex(rs_akc.serialize())
    assert.equal(aa, bb)
    let d_akc = AcknowledgementChallenge.deserialize(ts_akc.serialize())
    d_akc.validate(rs_hk.to_challenge(), pub_key_1)
    assert(d_akc.eq(rs_akc), "ack challenge serde test failed")
  })

  it('unacknowledged ticket', async function() {
    let challenge = Uint8Array.from(randomBytes(32))

    const user_1 = TsAddress.deserialize(pub_key_1.to_address().serialize())
    const challenge_1 = new TsResponse(challenge).toChallenge().toEthereumChallenge()
    const epoch_1 = UINT256.fromString('1')
    const index_1 = UINT256.fromString('1')
    const amount_1 = new TsBalance(new BN(1))
    const winProb_1 = UINT256.fromInverseProbability(new BN(1))
    const channelEpoch_1 = UINT256.fromString('1')
    const signature_1 = new TsSignature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    let ts_ticket = new TsTicket(user_1, challenge_1, epoch_1, index_1, amount_1, winProb_1, channelEpoch_1, signature_1)

    const user_2 = pub_key_1.to_address() as Address
    const challenge_2 = new Response(challenge).to_challenge().to_ethereum_challenge()
    const epoch_2 = U256.one()
    const index_2 = U256.one()
    const amount_2 = new Balance('1', BalanceType.HOPR)
    const winProb_2 = U256.from_inverse_probability(U256.one())
    const channelEpoch_2 = U256.one()
    const signature_2 = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    let rs_ticket = new Ticket(user_2, challenge_2, epoch_2, index_2, amount_2, winProb_2, channelEpoch_2, signature_2)

    let rs_hk = new HalfKey(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
    let ts_hk = new TsHalfKey(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))

    let rs_unack = new UnacknowledgedTicket(rs_ticket, rs_hk, pub_key_2)
    let ts_unack = new TsUnacknowledgedTicket(ts_ticket, ts_hk, TsPublicKey.fromPrivKey(private_key_2))

    let a = u8aToHex(rs_unack.serialize())
    let b = u8aToHex(ts_unack.serialize())

    assert.equal(a, b)
    assert(UnacknowledgedTicket.deserialize(ts_unack.serialize()).eq(rs_unack), "unack ticket serde test failed")
  })


})