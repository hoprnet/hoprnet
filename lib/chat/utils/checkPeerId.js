"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.checkPeerIdInput = void 0;
const chalk_1 = __importDefault(require("chalk"));
const peer_id_1 = __importDefault(require("peer-id"));
const multihashes_1 = __importDefault(require("multihashes"));
const bs58_1 = __importDefault(require("bs58"));
const utils_1 = require("../../src/utils");
/**
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param query query that contains the peerId
 */
async function checkPeerIdInput(query) {
    let peerId;
    try {
        // Throws an error if the Id is invalid
        multihashes_1.default.decode(bs58_1.default.decode(query));
        peerId = await utils_1.addPubKey(peer_id_1.default.createFromB58String(query));
    }
    catch (err) {
        throw Error(chalk_1.default.red(`Invalid peerId. ${err.message}`));
    }
    return peerId;
}
exports.checkPeerIdInput = checkPeerIdInput;
//# sourceMappingURL=checkPeerId.js.map