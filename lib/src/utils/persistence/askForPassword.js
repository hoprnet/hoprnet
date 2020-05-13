"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.askForPassword = void 0;
const read_1 = __importDefault(require("read"));
/**
 *
 * @param question question to ask before prompt
 */
function askForPassword(question) {
    if (process.env.DEVEVLOP_MODE === 'true') {
        console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St');
        return Promise.resolve('Epo5kZTFidOCHrnL0MzsXNwN9St');
    }
    return new Promise((resolve, reject) => {
        read_1.default({
            prompt: question + ' (Password will not be echoed.)\n  password:',
            silent: true,
            edit: true
        }, (err, pw) => {
            if (err) {
                return reject(err);
            }
            resolve(pw);
        });
    });
}
exports.askForPassword = askForPassword;
//# sourceMappingURL=askForPassword.js.map