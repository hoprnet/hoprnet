"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const extended_1 = require("../../types/extended");
const constants_1 = require("../../constants");
// @TODO: SIZE check on construction
class BYTES32 extends extended_1.Uint8ArrayE {
    static get SIZE() {
        return constants_1.HASH_LENGTH;
    }
}
exports.default = BYTES32;
//# sourceMappingURL=bytes32.js.map