"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = require("fs");
/**
 * Deletes recursively (and synchronously) all files in a directory.
 *
 * @param path the path to the directory
 *
 * @example
 *
 * ```javascript
 * clearDirectory('./db')
 * // deletes all files and subdirectories in `./db`
 * ```
 */
function clearDirectory(path) {
    let files = [];
    if (fs_1.existsSync(path)) {
        files = fs_1.readdirSync(path);
        files.forEach((file) => {
            const curPath = path + '/' + file;
            if (fs_1.lstatSync(curPath).isDirectory()) {
                // recurse
                clearDirectory(curPath);
            }
            else {
                // delete file
                fs_1.unlinkSync(curPath);
            }
        });
        fs_1.rmdirSync(path);
    }
}
exports.clearDirectory = clearDirectory;
