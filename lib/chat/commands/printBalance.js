"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
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
        const { paymentChannels } = this.node;
        const { Balance, NativeBalance } = paymentChannels.types;
        const balance = await paymentChannels.accountBalance.then(b => {
            return hopr_utils_1.moveDecimalPoint(b.toString(), Balance.DECIMALS * -1);
        });
        const nativeBalance = await paymentChannels.accountNativeBalance.then(b => {
            return hopr_utils_1.moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1);
        });
        console.log([
            `Account Balance: ${chalk_1.default.magenta(balance)} ${Balance.SYMBOL}`,
            `Account Native Balance: ${chalk_1.default.magenta(nativeBalance)} ${NativeBalance.SYMBOL}`,
        ].join('\n'));
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = PrintBalance;
