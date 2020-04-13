"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Parse JSON while recovering all Buffer elements
 * @param str JSON string
 */
function parseJSON(str) {
    return JSON.parse(str, (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data);
        }
        return value;
    });
}
exports.parseJSON = parseJSON;
