"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.displayHelp = void 0;
const cliOptions_1 = require("./cliOptions");
const FIRST_OPTION_OFFSET = 1;
const SECOND_OPTION_OFFSET = 5;
function displayHelp() {
    let firstOptionMaxLength = 0;
    let secondOptionMaxLength = 0;
    for (let i = 0; i < cliOptions_1.cli_options.length; i++) {
        if (cliOptions_1.cli_options[i][0] != null && cliOptions_1.cli_options[i][0].length > firstOptionMaxLength) {
            firstOptionMaxLength = cliOptions_1.cli_options[i][0].length;
        }
        if (cliOptions_1.cli_options[i][2] != null) {
            if (cliOptions_1.cli_options[i][1] != null && cliOptions_1.cli_options[i][1].length + cliOptions_1.cli_options[i][2].length > secondOptionMaxLength) {
                secondOptionMaxLength = cliOptions_1.cli_options[i][1].length + cliOptions_1.cli_options[i][2].length;
            }
        }
        else {
            if (cliOptions_1.cli_options[i][1] != null && cliOptions_1.cli_options[i][1].length > secondOptionMaxLength) {
                secondOptionMaxLength = cliOptions_1.cli_options[i][1].length;
            }
        }
    }
    let str = '';
    const offset = firstOptionMaxLength + FIRST_OPTION_OFFSET + secondOptionMaxLength + SECOND_OPTION_OFFSET;
    for (let i = 0; i < cliOptions_1.cli_options.length; i++) {
        str += (cliOptions_1.cli_options[i][0] || '').padEnd(firstOptionMaxLength + FIRST_OPTION_OFFSET, ' ');
        str += (cliOptions_1.cli_options[i][1] != null
            ? '[' + cliOptions_1.cli_options[i][1] + ']' + (cliOptions_1.cli_options[i][2] != null ? ' ' + cliOptions_1.cli_options[i][2] : '')
            : '').padEnd(secondOptionMaxLength + SECOND_OPTION_OFFSET, ' ');
        if (offset + cliOptions_1.cli_options[i][3].length > process.stdout.columns) {
            const words = cliOptions_1.cli_options[i][3].split(/\s+/);
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
                    str += '\n' + ''.padEnd(offset, ' ') + words[j];
                    length = words[j].length;
                }
                if (j < words.length - 1) {
                    str += ' ';
                    length++;
                }
            }
        }
        else {
            str += cliOptions_1.cli_options[i][3];
        }
        if (i < cliOptions_1.cli_options.length - 1) {
            str += '\n';
        }
    }
    console.log(str);
}
exports.displayHelp = displayHelp;
//# sourceMappingURL=displayHelp.js.map