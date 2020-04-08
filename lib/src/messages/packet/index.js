"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const secp256k1_1 = __importDefault(require("secp256k1"));
const bn_js_1 = __importDefault(require("bn.js"));
const chalk_1 = __importDefault(require("chalk"));
const RELAY_FEE = 10;
function fromWei(arg, unit) {
    return arg.toString();
}
const utils_1 = require("../../utils");
const header_1 = require("./header");
const challenge_1 = require("./challenge");
const message_1 = __importDefault(require("./message"));
/**
 * Encapsulates the internal representation of a packet
 */
class Packet extends Uint8Array {
    constructor(node, arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Packet.SIZE(node.paymentChannels));
        }
        else if (arr == null && struct != null) {
            super(utils_1.u8aConcat(struct.header, struct.ticket, struct.challenge, struct.message));
        }
        else {
            throw Error(`Invalid constructor parameters.`);
        }
        this.node = node;
    }
    subarray(begin = 0, end = Packet.SIZE(this.node.paymentChannels)) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin);
    }
    get header() {
        if (this._header == null) {
            this._header = new header_1.Header({ bytes: this.buffer, offset: this.byteOffset });
        }
        return this._header;
    }
    get ticket() {
        if (this._ticket == null) {
            this._ticket = this.node.paymentChannels.types.SignedTicket.create({
                bytes: this.buffer,
                offset: this.byteOffset + header_1.Header.SIZE,
            });
        }
        return this._ticket;
    }
    get challenge() {
        if (this._challenge == null) {
            this._challenge = new challenge_1.Challenge(this.node.paymentChannels, {
                bytes: this.buffer,
                offset: this.byteOffset + header_1.Header.SIZE + this.node.paymentChannels.types.SignedTicket.SIZE,
            });
        }
        return this._challenge;
    }
    get message() {
        if (this._message == null) {
            this._message = new message_1.default({
                bytes: this.buffer,
                offset: this.byteOffset + header_1.Header.SIZE + this.node.paymentChannels.types.SignedTicket.SIZE + challenge_1.Challenge.SIZE(this.node.paymentChannels),
            }, true);
        }
        return this._message;
    }
    //     node.paymentChannels,
    //     new Header(buf.subarray(0, HeaderSIZE)),
    //     Transaction.fromBuffer(buf.slice(HeaderSIZE, HeaderSIZE + Transaction.SIZE)),
    //     new Challenge(node.paymentChannels, buf.subarray(HeaderSIZE + Transaction.SIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE)),
    //     new Message(buf.subarray(HeaderSIZE + Transaction.SIZE + ChallengeSIZE, HeaderSIZE + Transaction.SIZE + ChallengeSIZE + MessageSIZE), true)
    static SIZE(hoprCoreConnector) {
        return header_1.Header.SIZE + hoprCoreConnector.types.SignedTicket.SIZE + challenge_1.Challenge.SIZE(hoprCoreConnector) + message_1.default.SIZE;
    }
    /**
     * Creates a new packet.
     *
     * @param node the node itself
     * @param msg the message that is sent through the network
     * @param path array of peerId that determines the route that
     * the packet takes
     */
    static async create(node, msg, path) {
        const { header, secrets, identifier } = await header_1.Header.create(node, path);
        console.log('---------- New Packet ----------');
        path.slice(0, Math.max(0, path.length - 1)).forEach((peerId, index) => console.log(`Intermediate ${index} : ${chalk_1.default.blue(peerId.toB58String())}`));
        console.log(`Destination    : ${chalk_1.default.blue(path[path.length - 1].toB58String())}`);
        console.log('--------------------------------');
        const arr = new Uint8Array(Packet.SIZE(node.paymentChannels)).fill(0x00);
        const packet = new Packet(node, {
            bytes: arr.buffer,
            offset: arr.byteOffset,
        });
        const fee = new bn_js_1.default(secrets.length - 1, 10).imul(new bn_js_1.default(RELAY_FEE, 10));
        packet.header.set(header);
        packet.challenge.set(await challenge_1.Challenge.create(node.paymentChannels, await node.paymentChannels.utils.hash(header_1.deriveTicketKey(secrets[0])), fee).sign(node.peerInfo.id));
        packet.message.set(message_1.default.createPlain(msg).onionEncrypt(secrets));
        node.log(`Encrypting with ${node.paymentChannels.utils.hash(utils_1.u8aXOR(false, header_1.deriveTicketKey(secrets[0]), header_1.deriveTicketKey(secrets[1]))).toString()}.`);
        const channelBalance = node.paymentChannels.types.ChannelBalance.create(undefined, {
            balance: new bn_js_1.default(12345),
            balance_a: new bn_js_1.default(123),
        });
        const channel = await node.paymentChannels.channel.create(node.paymentChannels, path[0].pubKey.marshal(), (_counterparty) => node.interactions.payments.onChainKey.interact(path[0]), channelBalance, (_channelBalance) => node.interactions.payments.open.interact(path[0], channelBalance));
        packet.ticket.set(await channel.ticket.create(channel, fee, secp256k1_1.default.privateKeyTweakAdd(header_1.deriveTicketKey(secrets[0]), header_1.deriveTicketKey(secrets[1]))));
        return packet;
    }
    /**
     * Tries to get a previous transaction from the database. If there's no such one,
     * listen to the channel opening event for some time and throw an error if the
     * was not opened within `OPENING_TIMEOUT` ms.
     *
     * @param channelId ID of the channel
     */
    async getPreviousTransaction(channelId, state) {
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
     * @param node the node itself
     */
    async forwardTransform() {
        this.header.deriveSecret(this.node.peerInfo.id.privKey.marshal());
        if (await this.hasTag(this.node.db)) {
            throw Error('General error.');
        }
        if (!this.header.verify()) {
            throw Error('General error.');
        }
        this.header.extractHeaderInformation();
        const [sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()]);
        const senderAccountId = await this.node.paymentChannels.utils.pubKeyToAccountId(sender.pubKey.marshal());
        const channelId = await this.node.paymentChannels.utils.getId(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()), senderAccountId);
        // check if channel exists
        let isOpen = false;
        try {
            isOpen = await this.node.paymentChannels.channel.isOpen(this.node.paymentChannels, senderAccountId);
        }
        catch (err) {
            throw err;
        }
        if (!isOpen) {
            throw Error('Payment channel is not open');
        }
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
        this.node.log(`Payment channel exists. Requested SHA256 pre-image of ${chalk_1.default.green(this.node.paymentChannels.utils.hash(this.header.derivedSecret).toString())} is derivable.`);
        this.message.decrypt(this.header.derivedSecret);
        const challengeShadowCopy = this.challenge.getCopy();
        const oldChallenge = new challenge_1.Challenge(this.node.paymentChannels, {
            bytes: challengeShadowCopy.buffer,
            offset: challengeShadowCopy.byteOffset,
        });
        if (utils_1.u8aEquals(this.node.peerInfo.id.pubKey.marshal(), this.header.address)) {
            await this.prepareDelivery(null, null, channelId);
        }
        else {
            await this.prepareForward(null, null, target);
        }
        return oldChallenge;
    }
    /**
     * Prepares the delivery of the packet.
     *
     * @param node the node itself
     * @param state current off-chain state
     * @param newState future off-chain state
     * @param nextNode the ID of the payment channel
     */
    async prepareDelivery(state, newState, nextNode) {
        this.message.encrypted = false;
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
    async prepareForward(state, newState, target) {
        const channelId = await this.node.paymentChannels.utils.getId(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()), await this.node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()));
        await this.node.db.put(Buffer.from(this.node.dbKeys.UnAcknowledgedTickets(target.pubKey.marshal(), this.header.hashedKeyHalf)), Buffer.from(this.ticket));
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
        const channelBalance = this.node.paymentChannels.types.ChannelBalance.create(undefined, {
            balance: new bn_js_1.default(12345),
            balance_a: new bn_js_1.default(123),
        });
        const channel = await this.node.paymentChannels.channel.create(this.node.paymentChannels, target.pubKey.marshal(), (_counterparty) => this.node.interactions.payments.onChainKey.interact(target), channelBalance, (_channelBalance) => this.node.interactions.payments.open.interact(target, channelBalance));
        const receivedMoney = this.ticket.ticket.getEmbeddedFunds();
        this.node.log(`Received ${chalk_1.default.magenta(`${fromWei(receivedMoney, 'ether').toString()} ETH`)} on channel ${chalk_1.default.yellow(channelId.toString())}.`);
        // if (receivedMoney.lt(RELAY_FEE)) {
        //   throw Error('Bad transaction.')
        // }
        this.header.transformForNextNode();
        const forwardedFunds = receivedMoney.isub(new bn_js_1.default(RELAY_FEE, 10));
        this.challenge.set(await challenge_1.Challenge.create(this.node.paymentChannels, this.header.hashedKeyHalf, forwardedFunds).sign(this.node.peerInfo.id));
        // const [tx] = await Promise.all([
        //   node.paymentChannels.transfer(await node.paymentChannels.utils.pubKeyToAccountId(target.pubKey.marshal()), forwardedFunds, this.header.encryptionKey),
        //   node.paymentChannels.setState(channelId, newState),
        //   node.db
        //     .batch()
        //     .put(node.paymentChannels.dbKeys.ChannelId(await this.challenge.signatureHash), channelId)
        //     .put(node.paymentChannels.dbKeys.Challenge(channelId, this.header.hashedKeyHalf), deriveTicketKey(this.header.derivedSecret))
        //     .write()
        // ])
        this.ticket.set(await channel.ticket.create(channel, forwardedFunds, this.header.encryptionKey));
    }
    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     */
    async getTargetPeerId() {
        if (this._targetPeerId)
            return this._targetPeerId;
        this._targetPeerId = await utils_1.pubKeyToPeerId(this.header.address);
        return this._targetPeerId;
    }
    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     */
    async getSenderPeerId() {
        if (this._senderPeerId)
            return this._senderPeerId;
        this._senderPeerId = await utils_1.pubKeyToPeerId(await this.ticket.signer);
        return this._senderPeerId;
    }
    /**
     * Checks whether the packet has already been seen.
     */
    // @TODO: unhappy case mising
    async hasTag(db) {
        const tag = header_1.deriveTagParameters(this.header.derivedSecret);
        const key = Buffer.concat([Buffer.from('packet-tag-'), tag], 11 + 16);
        try {
            await db.get(key);
        }
        catch (err) {
            if (err.notFound != true) {
                this.node.log(err);
            }
            return false;
        }
        return true;
    }
}
exports.Packet = Packet;
