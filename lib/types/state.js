"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const _1 = require(".");
const extended_1 = require("../types/extended");
class State extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, State.SIZE);
        }
        else if (arr == null && struct != null) {
            super(hopr_utils_1.u8aConcat(struct.secret, struct.pubkey, struct.epoch.toU8a()));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get secret() {
        return new _1.Hash(this.subarray(0, _1.Hash.SIZE));
    }
    get pubkey() {
        return new _1.Public(this.subarray(_1.Hash.SIZE, _1.Hash.SIZE + _1.Public.SIZE));
    }
    get epoch() {
        return new _1.TicketEpoch(this.subarray(_1.Hash.SIZE + _1.Public.SIZE, _1.Hash.SIZE + _1.Public.SIZE + _1.TicketEpoch.SIZE));
    }
    static get SIZE() {
        return _1.Hash.SIZE + _1.Public.SIZE + _1.TicketEpoch.SIZE;
    }
}
exports.default = State;
//# sourceMappingURL=state.js.map