"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const bn_js_1 = __importDefault(require("bn.js"));
class PrintBalance {
    constructor(node) {
        this.node = node;
    }
    /**
     * Prints the balance of our account.
     * @notice triggered by the CLI
     */
    async execute() {
        console.log(`Account Balance: ${chalk_1.default.magenta((await this.node.paymentChannels.accountBalance).div(new bn_js_1.default(10).pow(new bn_js_1.default(this.node.paymentChannels.types.Balance.DECIMALS))).toString())} ${this.node.paymentChannels.types.Balance.SYMBOL}`);
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = PrintBalance;
