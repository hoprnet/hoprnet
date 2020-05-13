"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.PacketForwardInteraction = void 0;
const constants_1 = require("../../constants");
const packet_1 = require("../../messages/packet");
const acknowledgement_1 = require("../../messages/acknowledgement");
const peer_info_1 = __importDefault(require("peer-info"));
const chalk_1 = __importDefault(require("chalk"));
const abort_controller_1 = __importDefault(require("abort-controller"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_1 = require("../../utils");
const MAX_PARALLEL_JOBS = 20;
const TWO_SECONDS = 2 * 1000;
const FORWARD_TIMEOUT = TWO_SECONDS;
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
        const abort = new abort_controller_1.default();
        const signal = abort.signal;
        const timeout = setTimeout(() => {
            abort.abort();
        }, FORWARD_TIMEOUT);
        struct = await this.node
            .dialProtocol(counterparty, this.protocols[0], { signal })
            .catch(async (err) => {
            const peerInfo = await this.node.peerRouting.findPeer(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty);
            try {
                let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal });
                clearTimeout(timeout);
                return result;
            }
            catch (err) {
                clearTimeout(timeout);
                this.node.log(`Could not transfer packet to ${(peer_info_1.default.isPeerInfo(counterparty)
                    ? counterparty.id
                    : counterparty).toB58String()}. Error was: ${chalk_1.default.red(err.message)}.`);
                return;
            }
        });
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
                    offset: arr.byteOffset,
                });
                this.queue.push(packet);
                if (this.tokens.length > 0) {
                    const token = this.tokens.pop();
                    if (this.promises[token] != null) {
                        /**
                         * @TODO remove this and make sure that the Promise is always
                         * already resolved.
                         */
                        await this.promises[token];
                        this.promises[token] = this.handlePacket(token);
                    }
                    else {
                        this.handlePacket(token);
                    }
                }
            }
        });
    }
    async handlePacket(token) {
        let packet;
        let sender, target;
        // Check for unserviced packets
        while (this.queue.length > 0) {
            // Pick a random one
            const index = hopr_utils_1.randomInteger(0, this.queue.length);
            if (index == this.queue.length - 1) {
                packet = this.queue.pop();
            }
            else {
                packet = this.queue[index];
                this.queue[index] = this.queue.pop();
            }
            let { receivedChallenge, ticketKey } = await packet.forwardTransform();
            [sender, target] = await Promise.all([
                /* prettier-ignore */
                packet.getSenderPeerId(),
                packet.getTargetPeerId(),
            ]);
            setImmediate(async () => {
                const ack = new acknowledgement_1.Acknowledgement(this.node.paymentChannels, undefined, {
                    key: ticketKey,
                    challenge: receivedChallenge,
                });
                await this.node.interactions.packet.acknowledgment.interact(sender, await ack.sign(this.node.peerInfo.id));
            });
            if (this.node.peerInfo.id.isEqual(target)) {
                this.node.output(packet.message.plaintext);
            }
            else {
                await this.interact(target, packet);
            }
        }
        this.tokens.push(token);
    }
}
exports.PacketForwardInteraction = PacketForwardInteraction;
//# sourceMappingURL=forward.js.map