"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const solidity_1 = require("./solidity");
const web3_1 = __importDefault(require("web3"));
class AccountId extends solidity_1.BYTES32 {
    toHex() {
        return web3_1.default.utils.toChecksumAddress(super.toHex());
    }
}
exports.default = AccountId;
