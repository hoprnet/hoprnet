"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const extended_1 = require("../../types/extended");
class UINT256 extends extended_1.BNE {
    toU8a() {
        return super.toU8a(UINT256.SIZE);
    }
    static get SIZE() {
        return 32;
    }
}
exports.default = UINT256;
//# sourceMappingURL=uint256.js.map