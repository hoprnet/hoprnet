"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const web3_1 = __importDefault(require("web3"));
const solidity_1 = require("./solidity");
const constants_1 = require("../constants");
class AccountId extends solidity_1.BYTES32 {
    static get SIZE() {
        return constants_1.ADDRESS_LENGTH;
    }
    toHex() {
        return web3_1.default.utils.toChecksumAddress(super.toHex());
    }
}
exports.default = AccountId;
//# sourceMappingURL=accountId.js.map