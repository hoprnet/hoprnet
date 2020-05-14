"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.socketToConn = void 0;
const abortable_iterator_1 = __importDefault(require("abortable-iterator"));
const debug_1 = __importDefault(require("debug"));
const log = debug_1.default('libp2p:tcp:socket');
// @ts-ignore
const toIterable = require("stream-to-it");
const toMultiaddr = require('libp2p-utils/src/ip-port-to-multiaddr');
const constants_1 = require("./constants");
// Convert a socket into a MultiaddrConnection
// https://github.com/libp2p/interface-transport#multiaddrconnection
function socketToConn(socket, options) {
    options = options || {};
    // Check if we are connected on a unix path
    if (options.listeningAddr && options.listeningAddr.getPath()) {
        options.remoteAddr = options.listeningAddr;
    }
    if (options.remoteAddr && options.remoteAddr.getPath()) {
        options.localAddr = options.remoteAddr;
    }
    const { sink, source } = toIterable.duplex(socket);
    const maConn = {
        async sink(source) {
            if (options.signal) {
                source = abortable_iterator_1.default(source, options.signal);
            }
            try {
                await sink((async function* () {
                    for await (const chunk of source) {
                        // Convert BufferList to Buffer
                        yield Buffer.isBuffer(chunk) ? chunk : chunk.slice();
                    }
                })());
            }
            catch (err) {
                // If aborted we can safely ignore
                if (err.type !== 'aborted') {
                    // If the source errored the socket will already have been destroyed by
                    // toIterable.duplex(). If the socket errored it will already be
                    // destroyed. There's nothing to do here except log the error & return.
                    log(err);
                }
            }
        },
        source: options.signal ? abortable_iterator_1.default(source, options.signal) : source,
        conn: socket,
        localAddr: options.localAddr || toMultiaddr(socket.localAddress, socket.localPort),
        // If the remote address was passed, use it - it may have the peer ID encapsulated
        remoteAddr: options.remoteAddr || toMultiaddr(socket.remoteAddress, socket.remotePort),
        timeline: { open: Date.now() },
        close() {
            if (socket.destroyed)
                return;
            return new Promise((resolve, reject) => {
                const start = Date.now();
                // Attempt to end the socket. If it takes longer to close than the
                // timeout, destroy it manually.
                const timeout = setTimeout(() => {
                    const { host, port } = maConn.remoteAddr.toOptions();
                    log('timeout closing socket to %s:%s after %dms, destroying it manually', host, port, Date.now() - start);
                    if (socket.destroyed) {
                        log('%s:%s is already destroyed', host, port);
                    }
                    else {
                        socket.destroy();
                    }
                    resolve();
                }, constants_1.CLOSE_TIMEOUT);
                socket.once('close', () => clearTimeout(timeout));
                socket.end((err) => {
                    maConn.timeline.close = Date.now();
                    if (err)
                        return reject(err);
                    resolve();
                });
            });
        },
    };
    socket.once('close', () => {
        // In instances where `close` was not explicitly called,
        // such as an iterable stream ending, ensure we have set the close
        // timeline
        if (!maConn.timeline.close) {
            maConn.timeline.close = Date.now();
        }
    });
    return maConn;
}
exports.socketToConn = socketToConn;
//# sourceMappingURL=socket-to-conn.js.map