"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.displayHelp = void 0;
const __1 = require("..");
const FIRST_OPTION_OFFSET = 1;
const SECOND_OPTION_OFFSET = 5;
function displayHelp() {
    let firstOptionMaxLength = 0;
    let secondOptionMaxLength = 0;
    for (let i = 0; i < __1.cli_options.length; i++) {
        if (__1.cli_options[i][0] != null && __1.cli_options[i][0].length > firstOptionMaxLength) {
            firstOptionMaxLength = __1.cli_options[i][0].length;
        }
        if (__1.cli_options[i][2] != null) {
            if (__1.cli_options[i][1] != null &&
                __1.cli_options[i][1].length + __1.cli_options[i][2].length > secondOptionMaxLength) {
                secondOptionMaxLength = __1.cli_options[i][1].length + __1.cli_options[i][2].length;
            }
        }
        else {
            if (__1.cli_options[i][1] != null && __1.cli_options[i][1].length > secondOptionMaxLength) {
                secondOptionMaxLength = __1.cli_options[i][1].length;
            }
        }
    }
    let str = '';
    const offset = firstOptionMaxLength + FIRST_OPTION_OFFSET + secondOptionMaxLength + SECOND_OPTION_OFFSET;
    for (let i = 0; i < __1.cli_options.length; i++) {
        str += (__1.cli_options[i][0] || '').padEnd(firstOptionMaxLength + FIRST_OPTION_OFFSET, ' ');
        str += (__1.cli_options[i][1] != null
            ? '[' + __1.cli_options[i][1] + ']' + (__1.cli_options[i][2] != null ? ' ' + __1.cli_options[i][2] : '')
            : '').padEnd(secondOptionMaxLength + SECOND_OPTION_OFFSET, ' ');
        if (offset + __1.cli_options[i][3].length > process.stdout.columns) {
            const words = __1.cli_options[i][3].split(/\s+/);
            const allowance = process.stdout.columns - offset;
            let length = 0;
            for (let j = 0; j < words.length; j++) {
                if (words[j].length > allowance) {
                    str += words[j] + '\n';
                    continue;
                }
                if (length + words[j].length < allowance) {
                    str += words[j];
                    length += words[j].length;
                }
                else {
                    str +=
                        '\n' + ''.padEnd(offset, ' ') + words[j];
                    length = words[j].length;
                }
                if (j < words.length - 1) {
                    str += ' ';
                    length++;
                }
            }
        }
        else {
            str += __1.cli_options[i][3];
        }
        if (i < __1.cli_options.length - 1) {
            str += '\n';
        }
    }
    console.log(str);
}
exports.displayHelp = displayHelp;
//# sourceMappingURL=displayHelp.js.map