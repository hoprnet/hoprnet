"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const constants_1 = require("../../constants");
const packet_1 = require("../../messages/packet");
const acknowledgement_1 = require("../../messages/acknowledgement");
const peer_info_1 = __importDefault(require("peer-info"));
const chalk_1 = __importDefault(require("chalk"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const header_1 = require("../../messages/packet/header");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_1 = require("../../utils");
const MAX_PARALLEL_JOBS = 20;
class PacketForwardInteraction {
    constructor(node) {
        this.node = node;
        this.tokens = utils_1.getTokens(MAX_PARALLEL_JOBS);
        this.queue = [];
        this.promises = [];
        this.protocols = [constants_1.PROTOCOL_STRING];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    async interact(counterparty, packet) {
        let struct;
        try {
            struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err) => {
                return this.node.peerRouting
                    .findPeer(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty)
                    .then((peerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]));
            });
        }
        catch (err) {
            this.node.log(`Could not transfer packet to ${(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty).toB58String()}. Error was: ${chalk_1.default.red(err.message)}.`);
            return;
        }
        await it_pipe_1.default(
        /* prettier-ignore */
        [packet.subarray()], struct.stream);
    }
    handler(struct) {
        let packet;
        it_pipe_1.default(
        /* pretttier-ignore */
        struct.stream, async (source) => {
            for await (const msg of source) {
                const arr = msg.slice();
                packet = new packet_1.Packet(this.node, {
                    bytes: arr.buffer,
                    offset: arr.byteOffset
                });
                if (this.tokens.length > 0) {
                    const token = this.tokens.pop();
                    if (this.promises[token] != null) {
                        /**
                         * @TODO remove this and make sure that the Promise is always
                         * already resolved.
                         */
                        await this.promises[token];
                        this.promises[token] = this.handlePacket(packet, token);
                    }
                    else {
                        this.handlePacket(packet, token);
                    }
                }
                else {
                    this.queue.push(packet);
                }
            }
        });
    }
    // @TODO convert this into iterative function
    async handlePacket(packet, token) {
        const oldChallenge = await packet.forwardTransform();
        const [sender, target] = await Promise.all([
            /* prettier-ignore */
            packet.getSenderPeerId(),
            packet.getTargetPeerId()
        ]);
        // Acknowledgement
        setImmediate(async () => {
            const ack = new acknowledgement_1.Acknowledgement(this.node.paymentChannels, undefined, {
                key: header_1.deriveTicketKeyBlinding(packet.header.derivedSecret),
                challenge: oldChallenge
            });
            this.node.interactions.packet.acknowledgment.interact(sender, await ack.sign(this.node.peerInfo.id));
        });
        if (this.node.peerInfo.id.isEqual(target)) {
            this.node.output(packet.message.plaintext);
        }
        else {
            await this.interact(target, packet);
        }
        // Check for unserviced packets
        if (this.queue.length > 0) {
            // Pick a random one
            const index = hopr_utils_1.randomInteger(0, this.queue.length);
            if (index == this.queue.length - 1) {
                return this.handlePacket(this.queue.pop(), token);
            }
            const nextPacket = this.queue[index];
            this.queue[index] = this.queue.pop();
            return this.handlePacket(nextPacket, token);
        }
        this.tokens.push(token);
        return;
    }
}
exports.PacketForwardInteraction = PacketForwardInteraction;
