"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
/**
 * Starts an interval after a timeout.
 *
 * @param msg message to display
 */
function startDelayedInterval(msg) {
    let interval;
    let timeout = setTimeout(() => {
        process.stdout.write(`${chalk_1.default.green(msg)}\n`);
        interval = setInterval(() => {
            process.stdout.write(chalk_1.default.green('.'));
        }, 1000);
    }, 2 * 1000);
    return () => {
        clearTimeout(timeout);
        clearInterval(interval);
    };
}
exports.startDelayedInterval = startDelayedInterval;
