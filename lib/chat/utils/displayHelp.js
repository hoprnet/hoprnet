"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const __1 = require("..");
function displayHelp() {
    let length = 0;
    let currentLength;
    for (let i = 0; i < __1.cli_options.length; i++) {
        currentLength = __1.cli_options[i][0].length + (__1.cli_options[i][1] != null ? __1.cli_options[i][1].length : 0);
        if (currentLength > length) {
            length = currentLength;
        }
    }
    let str = '';
    for (let i = 0; i < __1.cli_options.length; i++) {
        if (__1.cli_options[i][1] != null) {
            str += (__1.cli_options[i][0] + ' [' + __1.cli_options[i][1] + ']').padEnd(length + 7, ' ');
        }
        else {
            str += __1.cli_options[i][0].padEnd(length + 7, ' ');
        }
        str += __1.cli_options[i][2];
        if (i < __1.cli_options.length - 1) {
            str += '\n';
        }
    }
    console.log(str);
}
exports.displayHelp = displayHelp;
