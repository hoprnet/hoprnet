"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const strip_ansi_1 = __importDefault(require("strip-ansi"));
const readline_1 = __importDefault(require("readline"));
/**
 * Takes a string that has been printed on the console and deletes
 * it line by line from the console.
 *
 * @notice Mainly used to get rid of questions printed to the console
 *
 * @param str string to delete
 * @param rl readline handle
 */
function clearString(str, rl) {
    const newLines = str.split(/\n/g);
    let lines = 0;
    let stripped;
    for (let i = 0; i < newLines.length; i++) {
        stripped = strip_ansi_1.default(newLines[i]);
        if (stripped.length > process.stdout.columns) {
            lines += Math.ceil(stripped.length / process.stdout.columns);
        }
        else {
            lines++;
        }
    }
    for (let i = 0; i < lines; i++) {
        readline_1.default.moveCursor(process.stdout, -rl.line, -1);
        readline_1.default.clearLine(process.stdout, 0);
    }
}
exports.clearString = clearString;
