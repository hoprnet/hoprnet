"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = require("fs");
/**
 * Creates a directory if it doesn't exist.
 *
 * @example
 * ```javascript
 * createDirectoryIfNotExists('db/testnet') // creates `./db` and `./db/testnet`
 * ```
 * @param path
 */
function createDirectoryIfNotExists(path) {
    const chunks = path.split('/');
    chunks.reduce((searchPath, chunk) => {
        searchPath += '/';
        searchPath += chunk;
        try {
            fs_1.accessSync(`${process.cwd()}${searchPath}`);
        }
        catch (err) {
            fs_1.mkdirSync(`${process.cwd()}${searchPath}`);
        }
        return searchPath;
    }, '');
}
exports.createDirectoryIfNotExists = createDirectoryIfNotExists;
