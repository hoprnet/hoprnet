"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const __1 = require("..");
const chalk_1 = __importDefault(require("chalk"));
class ListCommands {
    execute() {
        let maxLength = 0;
        for (let i = 0; i < __1.keywords.length; i++) {
            if (__1.keywords[i][0].length > maxLength) {
                maxLength = __1.keywords[i][0].length;
            }
        }
        let str = '';
        for (let i = 0; i < __1.keywords.length; i++) {
            str += chalk_1.default.yellow(('  ' + __1.keywords[i][0]).padEnd(maxLength + 6, ' '));
            str += __1.keywords[i][1];
            if (i < __1.keywords.length - 1) {
                str += '\n';
            }
        }
        console.log(str);
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = ListCommands;
