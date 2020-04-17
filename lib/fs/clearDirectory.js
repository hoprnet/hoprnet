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
    let files;
    if (!fs_1.existsSync(path)) {
        throw Error('Path does not exist.');
    }
    let toDelete = [path];
    let curPath;
    while (toDelete.length > 0) {
        curPath = toDelete[toDelete.length - 1];
        files = fs_1.readdirSync(curPath).map((file) => curPath + '/' + file);
        if (files.length > 0) {
            toDelete.push(...files);
        }
        else {
            toDelete.pop();
            if (fs_1.lstatSync(curPath).isDirectory()) {
                fs_1.rmdirSync(curPath);
            }
            else if (fs_1.lstatSync(curPath).isFile()) {
                fs_1.unlinkSync(curPath);
            }
            else {
                throw Error('not implemented');
            }
        }
    }
}
exports.clearDirectory = clearDirectory;
