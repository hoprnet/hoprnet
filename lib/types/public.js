"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const solidity_1 = require("./solidity");
const constants_1 = require("../constants");
const utils_1 = require("../utils");
class Public extends solidity_1.BYTES32 {
    get NAME() {
        return 'Public';
    }
    toAccountId() {
        return utils_1.pubKeyToAccountId(this);
    }
    static get SIZE() {
        return constants_1.COMPRESSED_PUBLIC_KEY_LENGTH;
    }
}
exports.default = Public;
//# sourceMappingURL=public.js.map