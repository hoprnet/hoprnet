import secp256k1 from 'secp256k1'
import BN from 'bn.js'
import chalk from 'chalk'

import PeerId from 'peer-id'
const RELAY_FEE = 10

function fromWei(arg: any, unit: any) {
  return arg.toString()
}
import { pubKeyToPeerId, u8aXOR, u8aConcat } from '../../utils'

import { Header, deriveTicketKey, deriveTagParameters } from './header'
import { Challenge } from './challenge'
import Message from './message'
import { LevelUp } from 'levelup'

import Hopr from '../../'

import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'

const PRIVATE_KEY_LENGTH = 32
const OPENING_TIMEOUT = 86400 * 1000

/**
 * Encapsulates the internal representation of a packet
 */
export class Packet<Chain extends HoprCoreConnectorInstance> extends Uint8Array {
  private _targetPeerId: PeerId
  private _senderPeerId: PeerId
  private oldChallenge: Challenge<Chain>

  hoprCoreConnector: Chain

  constructor(
    hoprCoreConnector: Chain,
    arr?: Uint8Array,
    struct?: {
      header: Header<Chain>
      ticket: Types.SignedTicket
      challenge: Challenge<Chain>
      message: Message
    }
  ) {
    if (arr != null && struct == null) {
      super(arr)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.header, struct.ticket, struct.challenge, struct.message))
    } else {
      throw Error(`Invalid constructor parameters.`)
    }

    this.hoprCoreConnector = hoprCoreConnector
  }


  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  get header(): Header<Chain> {
    return new Header<Chain>(this.subarray(0, Header.SIZE))
  }

  get ticket(): Types.SignedTicket {
    return new this.hoprCoreConnector.types.SignedTicket(this.subarray(Header.SIZE, Header.SIZE + this.hoprCoreConnector.types.Ticket.SIZE))
  }

  get challenge(): Challenge<Chain> {
    return new Challenge<Chain>(
      this.hoprCoreConnector,
      this.subarray(
        Header.SIZE + this.hoprCoreConnector.types.Ticket.SIZE,
        Header.SIZE + this.hoprCoreConnector.types.Ticket.SIZE + Challenge.SIZE(this.hoprCoreConnector)
      )
    )
  }

  get message(): Message {
    return new Message(
      this.subarray(
        Header.SIZE + this.hoprCoreConnector.types.Ticket.SIZE + Challenge.SIZE(this.hoprCoreConnector),
        Header.SIZE + this.hoprCoreConnector.types.Ticket.SIZE + Challenge.SIZE(this.hoprCoreConnector) + Message.SIZE
      ),
      true
    )
  }
  //     node.paymentChannels,
  //     new Header(buf.subarray(0, HeaderSIZE)),
  //     Transaction.fromBuffer(buf.slice(HeaderSIZE, HeaderSIZE + Transaction.SIZE)),
  //     new Challenge(node.paymentChannels, buf.subarray(HeaderSIZE + Transaction.SIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE)),
  //     new Message(buf.subarray(HeaderSIZE + Transaction.SIZE + ChallengeSIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE + MessageSIZE), true)

  static SIZE<Chain extends HoprCoreConnectorInstance>(hoprCoreConnector: Chain) {
    return Header.SIZE + hoprCoreConnector.types.SignedTicket.length + Challenge.SIZE(hoprCoreConnector) + Message.SIZE
  }

  /**
   * Creates a new packet.
   *
   * @param node the node itself
   * @param msg the message that is sent through the network
   * @param {PeerId[]} path array of peerId that determines the route that
   * the packet takes
   */
  static async create<Chain extends HoprCoreConnectorInstance>(node: Hopr<Chain>, msg: Uint8Array, path: PeerId[]) {
    const { header, secrets, identifier } = Header.create(path)

    node.log('---------- New Packet ----------')
    path.slice(0, Math.max(0, path.length - 1)).forEach((peerId, index) => node.log(`Intermediate ${index} : ${chalk.blue(peerId.toB58String())}`))
    node.log(`Destination    : ${chalk.blue(path[path.length - 1].toB58String())}`)
    node.log('--------------------------------')

    const fee = new BN(secrets.length - 1, 10).imul(new BN(RELAY_FEE, 10))

    console.log(deriveTicketKey(secrets[0]))
    const challenge = await Challenge.create(node.paymentChannels, await node.paymentChannels.utils.hash(deriveTicketKey(secrets[0])), fee).sign(
      node.peerInfo.id
    )

    const message = Message.createPlain(msg).onionEncrypt(secrets)

    node.log(`Encrypting with ${node.paymentChannels.utils.hash(u8aXOR(false, deriveTicketKey(secrets[0]), deriveTicketKey(secrets[1]))).toString()}.`)

    const channelBalance = new node.paymentChannels.types.ChannelBalance(node.paymentChannels, {
      balance: new BN(0),
      balance_a: new BN(123)
    })

    console.log(`node`, node)
    const channel = await node.paymentChannels.channel.create(
      node.paymentChannels,
      path[0].pubKey.marshal(),
      (_counterparty: Uint8Array) => node.interactions.payments.onChainKey.interact(path[0]),
      channelBalance,
      (_channelBalance: Types.ChannelBalance) => node.interactions.payments.open.interact(path[0], channelBalance)
    )

    const ticket = await channel.ticket.create(
      channel,
      fee,
      secp256k1.privateKeyTweakAdd(Buffer.from(deriveTicketKey(secrets[0])), Buffer.from(deriveTicketKey(secrets[1]))),
      node.peerInfo.id.privKey.marshal(),
      node.peerInfo.id.pubKey.marshal()
    )

    return new Packet(node.paymentChannels, null, {
      header,
      ticket,
      challenge,
      message
    })
  }

  /**
   * Tries to get a previous transaction from the database. If there's no such one,
   * listen to the channel opening event for some time and throw an error if the
   * was not opened within `OPENING_TIMEOUT` ms.
   *
   * @param channelId ID of the channel
   */
  async getPreviousTransaction(node: Hopr<Chain>, channelId: Uint8Array, state) {
    // const recordState = node.paymentChannels.TransactionRecordState
    // switch (state.state) {
    //   case recordState.OPENING:
    //     state = await new Promise((resolve, reject) =>
    //       setTimeout(
    //         (() => {
    //           const eventListener = node.paymentChannels.onceOpened(channelId, resolve)
    //           return () => {
    //             eventListener.removeListener(resolve)
    //             reject(Error(`Sender didn't send payment channel opening request for channel ${chalk.yellow(channelId.toString())} in time.`))
    //           }
    //         })(),
    //         OPENING_TIMEOUT
    //       )
    //     )
    //   case recordState.OPEN:
    //     return state.lastTransaction
    //   default:
    //     throw Error(`Invalid state of payment channel ${chalk.yellow(channelId.toString())}. Got '${state.state}'`)
    // }
  }

  /**
   * Checks the packet and transforms it such that it can be send to the next node.
   *
   * @param {Hopr} node the node itself
   */
  async forwardTransform(node: Hopr<Chain>): Promise<Packet<Chain>> {
    this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

    if (await this.hasTag(node.db)) {
      throw Error('General error.')
    }

    if (!this.header.verify()) {
      throw Error('General error.')
    }

    this.header.extractHeaderInformation()

    const [sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

    const channelId = await this.hoprCoreConnector.utils.getId(
      await this.hoprCoreConnector.utils.pubKeyToAccountId(node.peerInfo.id.pubKey.marshal()),
      await this.hoprCoreConnector.utils.pubKeyToAccountId(sender.pubKey.marshal())
    )

    // const currentState = await node.paymentChannels.state(channelId)

    // Update incoming payment channel
    // const newState = {
    //   currentOffchainBalance: this.transaction.value,
    //   currentIndex: this.transaction.index,
    //   lastTransaction: this.transaction
    // }

    // if (currentState.state == node.paymentChannels.TransactionRecordState.PRE_OPENED) {
    //   // @TODO da fehlt noch was
    //   node.log(`incoming payment over pre-opened channel ${chalk.yellow(channelId.toString())}`)
    //   newState.state = node.paymentChannels.TransactionRecordState.OPEN
    //   newState.nonce = this.transaction.nonce
    //   newState.counterparty = sender.pubKey.marshal()
    // } else {
    //   // Check whether we have an open channel in our database
    //   await this.getPreviousTransaction(node, channelId, currentState)
    // }

    // node.log(`Database index ${chalk.cyan(currentState.currentIndex.toString('hex'))} on channnel ${chalk.yellow(channelId.toString())}.`)
    // node.log(`Transaction index ${chalk.cyan(this.transaction.index.toString('hex'))} on channnel ${chalk.yellow(channelId.toString())}.`)

    // if (!new BN(currentState.currentIndex).addn(1).eq(new BN(this.transaction.index))) {
    //   throw Error('General error.')
    // }

    node.log(
      `Payment channel exists. Requested SHA256 pre-image of ${chalk.green(
        node.paymentChannels.utils.hash(this.header.derivedSecret).toString()
      )} is derivable.`
    )

    this.message.decrypt(this.header.derivedSecret)
    this.oldChallenge = this.challenge

    if (node.peerInfo.id.pubKey.marshal().every((value: number, index: number) => value == this.header.address[index])) {
      await this.prepareDelivery(node, null, null, channelId)
    } else {
      await this.prepareForward(node, null, null, target)
    }

    return this
  }

  /**
   * Prepares the delivery of the packet.
   *
   * @param node the node itself
   * @param state current off-chain state
   * @param newState future off-chain state
   * @param nextNode the ID of the payment channel
   */
  async prepareDelivery(node: Hopr<Chain>, state, newState, nextNode): Promise<void> {
    // const challenges = [secp256k1.publicKeyCreate(Buffer.from(deriveTicketKey(this.header.derivedSecret)))]
    // const previousChallenges = await (await node.paymentChannels.channel.create(node.paymentChannels, nextNode)).getPreviousChallenges()
    // if (previousChallenges != null) challenges.push(Buffer.from(previousChallenges))
    // if (state.channelKey) challenges.push(secp256k1.publicKeyCreate(state.channelKey))
    // if (!this.ticket.curvePoint.equals(secp256k1.publicKeyCombine(challenges))) {
    //   throw Error('General error.')
    // }
    // newState.channelKey = secp256k1.privateKeyTweakAdd(
    //   state.channelKey || Buffer.alloc(PRIVATE_KEY_LENGTH, 0),
    //   Buffer.from(deriveTicketKey(this.header.derivedSecret))
    // )
    // await node.paymentChannels.setState(nextNode, newState)
  }

  /**
   * Prepares the packet in order to forward it to the next node.
   *
   * @param node the node itself
   * @param state current off-chain state
   * @param newState future off-chain state
   * @param channelId the ID of the payment channel
   * @param target peer Id of the next node
   */
  async prepareForward(node: Hopr<Chain>, state, newState, target: PeerId): Promise<void> {
    const channelId = await this.hoprCoreConnector.utils.getId(
      await this.hoprCoreConnector.utils.pubKeyToAccountId(node.peerInfo.id.pubKey.marshal()),
      await this.hoprCoreConnector.utils.pubKeyToAccountId(target.pubKey.marshal())
    )

    // const challenges = [secp256k1.publicKeyCreate(Buffer.from(deriveTicketKey(this.header.derivedSecret))), this.header.hashedKeyHalf]
    // let previousChallenges = await (await node.paymentChannels.channel.create(node.paymentChannels, await node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()))).getPreviousChallenges()

    // if (previousChallenges != null) {
    //   challenges.push(previousChallenges)
    // }

    // if (state.channelKey) {
    //   challenges.push(secp256k1.publicKeyCreate(state.channelKey))
    // }

    // if (!this.ticket.curvePoint.equals(secp256k1.publicKeyCombine(challenges.map((challenge: Uint8Array) => Buffer.from(challenge))))) {
    //   throw Error('General error.')
    // }

    const channelBalance = new node.paymentChannels.types.ChannelBalance(node.paymentChannels, {
      balance: new BN(0),
      balance_a: new BN(123)
    })

    const channel = await node.paymentChannels.channel.create(
      node.paymentChannels,
      target.pubKey.marshal(),
      (_counterparty: Uint8Array) => node.interactions.payments.onChainKey.interact(target),
      channelBalance,
      (_channelBalance: Types.ChannelBalance) => node.interactions.payments.open.interact(target, channelBalance)
    )

    const receivedMoney = this.ticket.ticket.getEmbeddedFunds()

    node.log(`Received ${chalk.magenta(`${fromWei(receivedMoney, 'ether').toString()} ETH`)} on channel ${chalk.yellow(channelId.toString())}.`)

    // if (receivedMoney.lt(RELAY_FEE)) {
    //   throw Error('Bad transaction.')
    // }

    this.header.transformForNextNode()

    const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

    this.challenge.set(await Challenge.create<Chain>(this.hoprCoreConnector, this.header.hashedKeyHalf, forwardedFunds).sign(node.peerInfo.id))

    // const [tx] = await Promise.all([
    //   node.paymentChannels.transfer(await node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()), forwardedFunds, this.header.encryptionKey),
    //   node.paymentChannels.setState(channelId, newState),
    //   node.db
    //     .batch()
    //     .put(node.paymentChannels.dbKeys.ChannelId(await this.challenge.signatureHash), channelId)
    //     .put(node.paymentChannels.dbKeys.Challenge(channelId, this.header.hashedKeyHalf), deriveTicketKey(this.header.derivedSecret))
    //     .write()
    // ])

    this.ticket.set(
      await channel.ticket.create(channel, forwardedFunds, this.header.encryptionKey, node.peerInfo.id.privKey.marshal(), node.peerInfo.id.pubKey.marshal())
    )
  }

  /**
   * Computes the peerId of the next downstream node and caches it for later use.
   */
  async getTargetPeerId(): Promise<PeerId> {
    if (this._targetPeerId) return this._targetPeerId

    this._targetPeerId = await pubKeyToPeerId(this.header.address)

    return this._targetPeerId
  }

  /**
   * Computes the peerId if the preceeding node and caches it for later use.
   */
  async getSenderPeerId(): Promise<PeerId> {
    if (this._senderPeerId) return this._senderPeerId

    this._senderPeerId = await pubKeyToPeerId(this.ticket.signer)

    return this._senderPeerId
  }

  /**
   * Checks whether the packet has already been seen.
   */
  async hasTag(db: LevelUp): Promise<boolean> {
    const tag = deriveTagParameters(this.header.derivedSecret)
    const key = Buffer.concat([Buffer.from('packet-tag-'), tag], 11 + 16)

    try {
      await db.get(key)
    } catch (err) {
      if (err.notFound) {
        return false
      }
      throw err
    }

    return true
  }

  /**
   * @returns the binary representation of the packet
   */
  // toBuffer(): Uint8Array {
  //   return u8aConcat(this.header, this.ticket, this.challenge, this.message)
  // }

  // static fromBuffer<Chain extends HoprCoreConnectorInstance>(node: Hopr<Chain>, buf: Uint8Array) {
  //   if (buf.length != Packet.SIZE(node.paymentChannels))
  //     throw Error(
  //       `Invalid input parameter. Expected a Buffer of size ${Packet.SIZE}. Got instead ${typeof buf}${
  //         Buffer.isBuffer(buf) ? ` of length ${buf.length} but expected length ${Packet.SIZE}` : ''
  //       }.`
  //     )

  //   return new Packet(
  //     node.paymentChannels,
  //     new Header(buf.subarray(0, HeaderSIZE)),
  //     Transaction.fromBuffer(buf.slice(HeaderSIZE, HeaderSIZE + Transaction.SIZE)),
  //     new Challenge(node.paymentChannels, buf.subarray(HeaderSIZE + Transaction.SIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE)),
  //     new Message(buf.subarray(HeaderSIZE + Transaction.SIZE + ChallengeSIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE + MessageSIZE), true)
  //   )
}
