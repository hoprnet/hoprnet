"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Heartbeat = void 0;
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const abort_controller_1 = __importDefault(require("abort-controller"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const constants_1 = require("../../constants");
const peer_info_1 = __importDefault(require("peer-info"));
const HASH_FUNCTION = 'blake2s256';
const TWO_SECONDS = 2 * 1000;
const HEARTBEAT_TIMEOUT = TWO_SECONDS;
class Heartbeat {
    constructor(node) {
        this.node = node;
        this.protocols = [constants_1.PROTOCOL_HEARTBEAT];
        this.node.handle(this.protocols, this.handler.bind(this));
    }
    handler(struct) {
        let events = this.node.network.heartbeat;
        it_pipe_1.default(struct.stream, (source) => {
            return (async function* () {
                for await (const msg of source) {
                    events.emit('beat', struct.connection.remotePeer);
                    yield crypto_1.createHash(HASH_FUNCTION).update(msg.slice()).digest();
                }
            })();
        }, struct.stream);
    }
    async interact(counterparty) {
        let struct;
        const abort = new abort_controller_1.default();
        const signal = abort.signal;
        const timeout = setTimeout(() => {
            abort.abort();
        }, HEARTBEAT_TIMEOUT);
        struct = await this.node.dialProtocol(counterparty, this.protocols[0], { signal }).catch(async (err) => {
            const peerInfo = await this.node.peerRouting.findPeer(peer_info_1.default.isPeerInfo(counterparty) ? counterparty.id : counterparty);
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
        const challenge = crypto_1.randomBytes(16);
        const expectedResponse = crypto_1.createHash(HASH_FUNCTION).update(challenge).digest();
        await it_pipe_1.default(
        /** prettier-ignore */
        [challenge], struct.stream, async (source) => {
            let done = false;
            for await (const msg of source) {
                if (done == true) {
                    continue;
                }
                if (hopr_utils_1.u8aEquals(msg, expectedResponse)) {
                    done = true;
                }
            }
        });
    }
}
exports.Heartbeat = Heartbeat;
//# sourceMappingURL=heartbeat.js.map