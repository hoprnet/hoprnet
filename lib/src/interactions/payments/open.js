"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Opening = void 0;
const it_pipe_1 = __importDefault(require("it-pipe"));
const constants_1 = require("../../constants");
const peer_info_1 = __importDefault(require("peer-info"));
class Opening {
    constructor(node) {
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_PAYMENT_CHANNEL];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    async handler(struct) {
        it_pipe_1.default(
        /** prettier-ignore */
        struct.stream, this.node.paymentChannels.channel.handleOpeningRequest(this.node.paymentChannels), struct.stream);
    }
    async interact(counterparty, channelBalance) {
        let struct;
        try {
            struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (_) => {
                return this.node.peerRouting
                    .findPeer(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty)
                    .then((peerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]));
            });
        }
        catch (err) {
            throw Error(`Tried to open a payment channel but could not connect to ${(peer_info_1.default.isPeerInfo(counterparty)
                ? counterparty.id
                : counterparty).toB58String()}. Error was: ${err.message}`);
        }
        return await it_pipe_1.default(
        /* prettier-ignore */
        [(await this.node.paymentChannels.types.SignedChannel.create(this.node.paymentChannels, undefined, { channel: this.node.paymentChannels.types.Channel.createFunded(channelBalance) })).subarray()], struct.stream, this.collect.bind(this));
    }
    async collect(source) {
        let result;
        for await (const msg of source) {
            if (result != null) {
                continue;
            }
            else {
                result = msg.slice();
            }
        }
        if (result == null) {
            throw Error('Empty stream');
        }
        return this.node.paymentChannels.types.SignedChannel.create(this.node.paymentChannels, {
            bytes: result.buffer,
            offset: result.byteOffset
        });
    }
}
exports.Opening = Opening;
//# sourceMappingURL=open.js.map