"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.decodeMessage = exports.encodeMessage = void 0;
const chalk_1 = __importDefault(require("chalk"));
const rlp_1 = require("rlp");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
/**
 * Adds the current timestamp to the message in order to measure the latency.
 * @param msg the message
 */
function encodeMessage(msg) {
    return rlp_1.encode([msg, Date.now()]);
}
exports.encodeMessage = encodeMessage;
/**
 * Tries to decode the message and returns the message as well as
 * the measured latency.
 * @param encoded an encoded message
 */
function decodeMessage(encoded) {
    let msg, time;
    try {
        ;
        [msg, time] = rlp_1.decode(encoded);
        return {
            latency: Date.now() - parseInt(time.toString('hex'), 16),
            msg: msg.toString()
        };
    }
    catch (err) {
        console.log(chalk_1.default.red(`Could not decode received message '${hopr_utils_1.u8aToHex(encoded)}' Error was ${err.message}.`));
        return {
            latency: NaN,
            msg: 'Error: Could not decode message'
        };
    }
}
exports.decodeMessage = decodeMessage;
//# sourceMappingURL=message.js.map