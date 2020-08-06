"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const solidity_1 = require("../types/solidity");
const extended_1 = require("../types/extended");
// @TODO: we should optimize this since it will use more storage than needed
class ChannelEntry extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, ChannelEntry.SIZE);
        }
        else if (arr == null && struct != null) {
            // we convert values to string because of this issue
            // https://github.com/indutny/bn.js/issues/206
            const blockNumber = new extended_1.BNE(struct.blockNumber.toString());
            const transactionIndex = new extended_1.BNE(struct.transactionIndex.toString());
            const logIndex = new extended_1.BNE(struct.logIndex.toString());
            super(hopr_utils_1.u8aConcat(blockNumber.toU8a(solidity_1.UINT256.SIZE), transactionIndex.toU8a(solidity_1.UINT256.SIZE), logIndex.toU8a(solidity_1.UINT256.SIZE)));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get blockNumber() {
        return new extended_1.BNE(this.subarray(0, solidity_1.UINT256.SIZE));
    }
    get transactionIndex() {
        return new extended_1.BNE(this.subarray(solidity_1.UINT256.SIZE, solidity_1.UINT256.SIZE * 2));
    }
    get logIndex() {
        return new extended_1.BNE(this.subarray(solidity_1.UINT256.SIZE * 2, solidity_1.UINT256.SIZE * 3));
    }
    static get SIZE() {
        return solidity_1.UINT256.SIZE * 3;
    }
}
exports.default = ChannelEntry;
//# sourceMappingURL=channelEntry.js.map