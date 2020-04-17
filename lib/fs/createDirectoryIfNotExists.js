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
    if (path.endsWith('/')) {
        path = path.substring(0, path.length - 1);
    }
    const chunks = path.split('/');
    let searchPath = '';
    for (let i = 0; i < chunks.length; i++) {
        searchPath += chunks[i] + '/';
        if (!fs_1.existsSync(searchPath)) {
            fs_1.mkdirSync(searchPath);
        }
    }
}
exports.createDirectoryIfNotExists = createDirectoryIfNotExists;
