"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const keywords_1 = require("../utils/keywords");
const chalk_1 = __importDefault(require("chalk"));
class ListCommands {
    execute() {
        let maxLength = 0;
        for (let i = 0; i < keywords_1.keywords.length; i++) {
            if (keywords_1.keywords[i][0].length > maxLength) {
                maxLength = keywords_1.keywords[i][0].length;
            }
        }
        let str = '';
        for (let i = 0; i < keywords_1.keywords.length; i++) {
            str += chalk_1.default.yellow(('  ' + keywords_1.keywords[i][0]).padEnd(maxLength + 6, ' '));
            str += keywords_1.keywords[i][1];
            if (i < keywords_1.keywords.length - 1) {
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
//# sourceMappingURL=listCommands.js.map