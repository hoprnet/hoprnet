"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ForwardPacketInteraction = void 0;
const utils_1 = require("../../utils");
const forward_1 = require("../../messages/forward");
const abort_controller_1 = __importDefault(require("abort-controller"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const it_pushable_1 = __importDefault(require("it-pushable"));
const constants_1 = require("../../constants");
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const MAX_PARALLEL_JOBS = 20;
const TWO_SECONDS = 2 * 1000;
const FORWARD_TIMEOUT = TWO_SECONDS;
class ForwardPacketInteraction {
    constructor(node) {
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_FORWARD];
        this.tokens = utils_1.getTokens(MAX_PARALLEL_JOBS);
        this.queue = [];
        this.promises = [];
        this.connectionEnds = new Map();
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    handler(struct) {
        let forwardPacket;
        it_pipe_1.default(
        /* prettier-ignore */
        struct.stream, async (source) => {
            for await (const msg of source) {
                const arr = msg.slice();
                forwardPacket = new forward_1.ForwardPacket({
                    bytes: arr.buffer,
                    offset: arr.byteOffset,
                });
                this.queue.push(forwardPacket);
                if (this.tokens.length > 0) {
                    const token = this.tokens.pop();
                    if (this.promises[token] != null) {
                        /**
                         * @TODO remove this and make sure that the Promise is always
                         * already resolved.
                         */
                        await this.promises[token];
                        this.promises[token] = this.handleForwardPacket(token);
                    }
                    else {
                        this.handleForwardPacket(token);
                    }
                }
            }
        });
    }
    async handleForwardPacket(token) {
        let struct;
        let destination;
        let sender;
        let forwardPacket;
        let abort;
        let signal;
        let timeout;
        while (this.queue.length > 0) {
            forwardPacket = this.queue.pop();
            destination = await peer_id_1.default.createFromPubKey(Buffer.from(forwardPacket.destination));
            if (this.node.peerInfo.id.isEqual(destination)) {
                sender = await peer_id_1.default.createFromPubKey(Buffer.from(forwardPacket.sender));
                let connectionEnd = this.connectionEnds.get(sender.toB58String());
                if (connectionEnd != null) {
                    connectionEnd.push(forwardPacket.payload);
                }
                else {
                    throw Error(`Received unexpected forwarded packet.`);
                }
                continue;
            }
            abort = new abort_controller_1.default();
            signal = abort.signal;
            timeout = setTimeout(() => {
                // @TODO add short-term storage here
                abort.abort();
            }, FORWARD_TIMEOUT);
            struct = await this.node
                .dialProtocol(destination, this.protocols[0], { signal })
                .catch(async (err) => {
                const peerInfo = await this.node.peerRouting.findPeer(destination);
                try {
                    let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal });
                    clearTimeout(timeout);
                    return result;
                }
                catch (err) {
                    clearTimeout(timeout);
                    throw err;
                }
            });
            await it_pipe_1.default(
            /* prettier-ignore */
            [forwardPacket], struct.stream);
        }
        this.tokens.push(token);
    }
    async interact(counterparty, relay) {
        let struct;
        let relayPeerId = peer_info_1.default.isPeerInfo(relay) ? relay.id : relay;
        let counterpartyPeerId = peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty;
        const abort = new abort_controller_1.default();
        const signal = abort.signal;
        const timeout = setTimeout(() => {
            abort.abort();
        }, FORWARD_TIMEOUT);
        struct = await this.node
            .dialProtocol(relay, this.protocols[0], { signal })
            .catch(async (err) => {
            const peerInfo = await this.node.peerRouting.findPeer(relayPeerId);
            try {
                let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal });
                clearTimeout(timeout);
                return result;
            }
            catch (err) {
                clearTimeout(timeout);
                throw err;
            }
        });
        const connectionEnd = it_pushable_1.default();
        this.connectionEnds.set(counterpartyPeerId.toB58String(), connectionEnd);
        let self = this;
        return {
            source: connectionEnd,
            sink: async function (source) {
                it_pipe_1.default(
                /* prettier-ignore */
                source, (source) => {
                    return (async function* () {
                        for await (let msg of source) {
                            yield new forward_1.ForwardPacket(undefined, {
                                destination: counterpartyPeerId,
                                sender: self.node.peerInfo.id,
                                payload: msg.slice(),
                            });
                        }
                    })();
                }, struct.stream);
            },
        };
    }
}
exports.ForwardPacketInteraction = ForwardPacketInteraction;
//# sourceMappingURL=forwardPacket.js.map