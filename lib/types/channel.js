"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ChannelStatus = void 0;
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const _1 = require(".");
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
var ChannelStatus;
(function (ChannelStatus) {
    ChannelStatus[ChannelStatus["UNINITIALISED"] = 0] = "UNINITIALISED";
    ChannelStatus[ChannelStatus["FUNDING"] = 1] = "FUNDING";
    ChannelStatus[ChannelStatus["OPEN"] = 2] = "OPEN";
    ChannelStatus[ChannelStatus["PENDING"] = 3] = "PENDING";
})(ChannelStatus = exports.ChannelStatus || (exports.ChannelStatus = {}));
class Channel extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Channel.SIZE);
        }
        else if (arr == null && struct != null) {
            super(hopr_utils_1.u8aConcat(struct.balance.toU8a(), new Uint8Array([struct.status])));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get balance() {
        const balance = this.subarray(0, _1.ChannelBalance.SIZE);
        return new _1.ChannelBalance({
            bytes: balance.buffer,
            offset: balance.byteOffset,
        });
    }
    get stateCounter() {
        return Number(this.subarray(_1.ChannelBalance.SIZE, _1.ChannelBalance.SIZE + 1)[0]);
    }
    get status() {
        return utils_1.stateCountToStatus(this.stateCounter);
    }
    get hash() {
        return utils_1.hash(this);
    }
    async sign(privKey, pubKey, arr) {
        return await utils_1.sign(await this.hash, privKey, undefined, arr);
    }
    static get SIZE() {
        return _1.ChannelBalance.SIZE + 1;
    }
    static createFunded(balance) {
        return new Channel(undefined, {
            balance,
            status: ChannelStatus.FUNDING,
        });
    }
    static createActive(balance) {
        return new Channel(undefined, {
            balance,
            status: ChannelStatus.OPEN,
        });
    }
    static createPending(moment, balance) {
        return new Channel(undefined, {
            balance,
            status: ChannelStatus.PENDING,
            moment,
        });
    }
}
exports.default = Channel;
//# sourceMappingURL=channel.js.map