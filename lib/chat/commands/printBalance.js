"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
class PrintBalance {
    constructor(node) {
        this.node = node;
    }
    /**
     * Prints the balance of our account.
     * @notice triggered by the CLI
     */
    async execute() {
        // @TODO replace HOPR tokens by TOKEN_NAME
        console.log(`Account Balance:  `, chalk_1.default.magenta((await this.node.paymentChannels.accountBalance).toString()), `HOPR tokens`);
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = PrintBalance;
