import { Ticket, PublicKey, Balance } from '@hoprnet/hopr-core-ethereum'
import type HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { Challenge } from './challenge'
import {
  getPacketLength,
  POR_STRING_LENGTH,
  u8aSplit,
  deriveAckKeyShare,
  createPacket,
  forwardTransform,
  generateKeyShares,
  createPoRString,
  createFirstChallenge,
  preVerify
} from '@hoprnet/hopr-utils'
import type PeerId from 'peer-id'
import { publicKeyCreate } from 'secp256k1'
import BN from 'bn.js'
import { LevelUp } from 'levelup'
import { checkPacketTag } from '../../dbKeys'

export const MAX_HOPS = 3

const PACKET_LENGTH = getPacketLength(MAX_HOPS, POR_STRING_LENGTH, 0)

export class Packet {
  public isReceiver: boolean
  public isReadyToForward: boolean

  public plaintext: Uint8Array

  public packetTag: Uint8Array
  public nextHop: Uint8Array
  public ownShare: Uint8Array
  public ownKey: Uint8Array
  public nextChallenge: Uint8Array
  public nextAckChallenge: Uint8Array

  private constructor(private packet: Uint8Array, private challenge: Challenge, private ticket: Ticket) {}

  private setReadyToForward() {
    this.isReadyToForward = true

    return this
  }

  private setFinal(plaintext: Uint8Array, packetTag: Uint8Array) {
    this.packetTag = packetTag
    this.isReceiver = true
    this.isReadyToForward = false
    this.plaintext = plaintext

    return this
  }

  private setForward(
    ownKey: Uint8Array,
    ownShare: Uint8Array,
    nextHop: Uint8Array,
    nextChallenge: Uint8Array,
    nextAckChallenge: Uint8Array,
    packetTag: Uint8Array
  ) {
    this.isReceiver = false
    this.isReadyToForward = false

    this.ownKey = ownKey
    this.ownShare = ownShare
    this.nextHop = nextHop
    this.nextChallenge = nextChallenge
    this.nextAckChallenge = nextAckChallenge
    this.packetTag = packetTag

    return this
  }

  static async create(msg: Uint8Array, path: PeerId[], privKey: PeerId, chain: HoprCoreEthereum): Promise<Packet> {
    const { alpha, secrets } = generateKeyShares(path)

    const { ackChallenge, ticketChallenge } = createFirstChallenge(secrets)

    const porStrings: Uint8Array[] = []

    for (let i = 0; i < path.length - 1; i++) {
      porStrings.push(createPoRString(secrets.slice(i)))
    }

    const challenge = Challenge.create(ackChallenge, privKey)

    const self = new PublicKey(privKey.pubKey.marshal())
    const nextPeer = new PublicKey(path[0].pubKey.marshal())

    const packet = createPacket(secrets, alpha, msg, path, MAX_HOPS, POR_STRING_LENGTH, porStrings)

    const channel = new chain.channel(chain, self, nextPeer)

    const ticket = await channel.createTicket(new Balance(new BN(0)), new PublicKey(ticketChallenge), 1)

    return new Packet(packet, challenge, ticket).setReadyToForward()
  }

  serialize(): Uint8Array {
    return Uint8Array.from([...this.packet, ...this.challenge.serialize(), ...this.ticket.serialize()])
  }

  static get SIZE() {
    return PACKET_LENGTH + Challenge.SIZE + Ticket.SIZE
  }

  static deserialize(preArray: Uint8Array, privKey: PeerId, pubKeySender: PeerId): Packet {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (preArray.length != Packet.SIZE) {
      throw Error(`Invalid arguments`)
    }

    let arr: Uint8Array
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(preArray)) {
      arr = Uint8Array.from(arr)
    } else {
      arr = preArray
    }

    const [packet, preChallenge, preTicket] = u8aSplit(arr, [PACKET_LENGTH, Challenge.SIZE, Ticket.SIZE])

    const transformedOutput = forwardTransform(privKey, packet, POR_STRING_LENGTH, 0, MAX_HOPS)

    transformedOutput.packetTag
    const ackKey = deriveAckKeyShare(transformedOutput.derivedSecret)

    const challenge = Challenge.deserialize(preChallenge, publicKeyCreate(ackKey), pubKeySender)

    const ticket = Ticket.deserialize(preTicket)

    if (transformedOutput.lastNode == true) {
      return new Packet(packet, challenge, ticket).setFinal(transformedOutput.plaintext, transformedOutput.packetTag)
    }

    const verificationOutput = preVerify(
      transformedOutput.derivedSecret,
      transformedOutput.additionalRelayData,
      ticket.challenge.serialize()
    )

    if (verificationOutput.valid != true) {
      throw Error(`General error.`)
    }

    return new Packet(transformedOutput.packet, challenge, ticket).setForward(
      verificationOutput.ownKey,
      verificationOutput.ownShare,
      transformedOutput.nextHop,
      verificationOutput.nextTicketChallenge,
      verificationOutput.nextAckChallenge,
      transformedOutput.packetTag
    )
  }

  async checkPacketTag(db: LevelUp) {
    const tagValid = await checkPacketTag(db, this.packetTag)

    if (!tagValid) {
      throw Error(`General error.`)
    }
  }

  async forwardTransform(privKey: PeerId, chain: HoprCoreEthereum): Promise<void> {
    if (privKey.privKey == null) {
      throw Error(`Invalid arguments`)
    }

    if (this.isReceiver || this.isReadyToForward) {
      throw Error(`Invalid state`)
    }

    const self = new PublicKey(privKey.pubKey.marshal())
    const nextPeer = new PublicKey(this.nextHop)

    const channel = new chain.channel(chain, self, nextPeer)

    this.ticket = await channel.createTicket(new Balance(new BN(0)), new PublicKey(this.nextChallenge), 0)

    this.challenge = Challenge.create(this.nextAckChallenge, privKey)

    this.isReadyToForward = true
  }
}
