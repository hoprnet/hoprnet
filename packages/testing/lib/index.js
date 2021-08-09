"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getNewPort = exports.Ganache = void 0;
var ganache_1 = require("./ganache");
Object.defineProperty(exports, "Ganache", { enumerable: true, get: function () { return __importDefault(ganache_1).default; } });
let port = 64000; // Use ports in 64XXX range.
function getNewPort() {
    if (port < 65535) {
        return port++;
    }
    throw new Error('Out of valid ports');
}
exports.getNewPort = getNewPort;
//# sourceMappingURL=index.js.map