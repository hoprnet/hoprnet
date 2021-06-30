"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_core_1 = __importDefault(require("@hoprnet/hopr-core"));
async function main() {
    const peerId = null;
    const options = {};
    const node = new hopr_core_1.default(peerId, options);
    await node.waitForFunds();
    await node.start();
}
main();
//# sourceMappingURL=index.js.map