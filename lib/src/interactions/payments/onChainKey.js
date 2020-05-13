"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.OnChainKey = void 0;
const constants_1 = require("../../constants");
const peer_info_1 = __importDefault(require("peer-info"));
const chalk_1 = __importDefault(require("chalk"));
const it_pipe_1 = __importDefault(require("it-pipe"));
class OnChainKey {
    constructor(node) {
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_ONCHAIN_KEY];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    handler(struct) {
        it_pipe_1.default(
        /* prettier-ignore */
        [this.node.paymentChannels.self.onChainKeyPair.publicKey], struct.stream);
    }
    async interact(counterparty) {
        let struct;
        try {
            struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (_) => {
                return this.node.peerRouting
                    .findPeer(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty)
                    .then((peerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]));
            });
        }
        catch (err) {
            throw Error(`Tried to get onChain key from party ${(peer_info_1.default.isPeerInfo(counterparty)
                ? counterparty.id
                : counterparty).toB58String()} but failed while trying to connect to that node. Error was: ${chalk_1.default.red(err.message)}`);
        }
        return it_pipe_1.default(
        /* prettier-ignore */
        struct.stream, onReception);
    }
}
exports.OnChainKey = OnChainKey;
async function onReception(source) {
    let result;
    for await (const msg of source) {
        if (msg == null || msg.length == 0) {
            throw Error(`received ${msg} but expected a public key`);
        }
        if (result != null) {
            // ignore any further messages
            continue;
        }
        else {
            result = msg.slice();
        }
    }
    return result;
}
//# sourceMappingURL=onChainKey.js.map