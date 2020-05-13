"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CrawlResponse = void 0;
const _1 = require(".");
const rlp_1 = require("rlp");
const utils_1 = require("../../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const ENUM_LENGTH = 1;
class CrawlResponse extends Uint8Array {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr);
        }
        else if (arr == null && struct != null) {
            if (struct.peerInfos == null) {
                if (struct.status == _1.CrawlStatus.OK) {
                    throw Error(`Cannot have successful crawling responses without any peerInfos.`);
                }
                super(hopr_utils_1.u8aConcat(hopr_utils_1.toU8a(struct.status, ENUM_LENGTH)));
            }
            else if (struct.status == _1.CrawlStatus.OK) {
                super(hopr_utils_1.u8aConcat(hopr_utils_1.toU8a(struct.status, ENUM_LENGTH), rlp_1.encode(struct.peerInfos.map((peerInfo) => utils_1.serializePeerInfo(peerInfo)))));
            }
            else {
                throw Error(`Invalid creation parameters.`);
            }
        }
    }
    subarray(begin = 0, end) {
        return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined);
    }
    get statusRaw() {
        return this.subarray(0, ENUM_LENGTH);
    }
    get status() {
        return hopr_utils_1.u8aToNumber(this.statusRaw);
    }
    get peerInfosRaw() {
        return this.subarray(ENUM_LENGTH, this.length);
    }
    get peerInfos() {
        return Promise.all(rlp_1.decode(this.peerInfosRaw).map((arr) => utils_1.deserializePeerInfo(arr)));
    }
}
exports.CrawlResponse = CrawlResponse;
//# sourceMappingURL=resonse.js.map