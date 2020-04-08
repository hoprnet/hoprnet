"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const solidity_1 = require("./solidity");
class Balance extends solidity_1.UINT256 {
    static get SYMBOL() {
        return `HOPR`;
    }
    static get DECIMALS() {
        return 18;
    }
}
exports.default = Balance;
//# sourceMappingURL=balance.js.map