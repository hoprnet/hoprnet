"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const multiaddr_1 = __importDefault(require("multiaddr"));
function relayToConn(options) {
    return {
        ...options.stream,
        conn: options.stream,
        remoteAddr: multiaddr_1.default(`/p2p/${options.counterparty.toB58String()}`),
        async close(err) { },
        timeline: {
            open: Date.now(),
        },
    };
}
exports.relayToConn = relayToConn;
//# sourceMappingURL=relay-to-conn.js.map