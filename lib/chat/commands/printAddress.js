"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
class PrintAddress {
    constructor(node) {
        this.node = node;
    }
    /**
     * Prints the name of the network we are using and the
     * identity that we have on that chain.
     * @notice triggered by the CLI
     */
    async execute() {
        const prefixLength = Math.max(this.node.paymentChannels.constants.CHAIN_NAME.length, 'HOPR'.length) + 3;
        console.log(`${(this.node.paymentChannels.constants.CHAIN_NAME + ':').padEnd(prefixLength, ' ')}${chalk_1.default.green(hopr_utils_1.u8aToHex(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal())))}\n` +
            /* prettier-ignore */
            `${'HOPR:'.padEnd(prefixLength, ' ')}${chalk_1.default.green(this.node.peerInfo.id.toB58String())}`);
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = PrintAddress;
