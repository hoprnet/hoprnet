"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const solidity_1 = require("./solidity");
class NativeBalance extends solidity_1.UINT256 {
    static get SYMBOL() {
        return `ETH`;
    }
    static get DECIMALS() {
        return 18;
    }
}
exports.default = NativeBalance;
//# sourceMappingURL=nativeBalance.js.map