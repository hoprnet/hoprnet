"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.createListener = void 0;
const net_1 = __importDefault(require("net"));
const events_1 = __importDefault(require("events"));
const debug_1 = __importDefault(require("debug"));
const log = debug_1.default('libp2p:tcp:listener');
const error = debug_1.default('libp2p:tcp:listener:error');
const socket_to_conn_1 = require("./socket-to-conn");
const constants_1 = require("./constants");
const utils_1 = require("./utils");
/**
 * Attempts to close the given maConn. If a failure occurs, it will be logged.
 * @private
 * @param maConn
 */
async function attemptClose(maConn) {
    try {
        maConn && (await maConn.close());
    }
    catch (err) {
        error('an error occurred closing the connection', err);
    }
}
function createListener({ handler, upgrader }, options) {
    const listener = new events_1.default();
    const server = net_1.default.createServer(async (socket) => {
        // Avoid uncaught errors caused by unstable connections
        socket.on('error', err => log('socket error', err));
        let maConn;
        let conn;
        try {
            maConn = socket_to_conn_1.socketToConn(socket, { listeningAddr });
            log('new inbound connection %s', maConn.remoteAddr);
            conn = await upgrader.upgradeInbound(maConn);
        }
        catch (err) {
            error('inbound connection failed', err);
            return attemptClose(maConn);
        }
        log('inbound connection %s upgraded', maConn.remoteAddr);
        trackConn(server, maConn);
        if (handler) {
            handler(conn);
        }
        listener.emit('connection', conn);
    });
    server
        .on('listening', () => listener.emit('listening'))
        .on('error', err => listener.emit('error', err))
        .on('close', () => listener.emit('close'));
    // Keep track of open connections to destroy in case of timeout
    // @ts-ignore
    server.__connections = [];
    listener.close = () => {
        if (!server.listening)
            return;
        return new Promise((resolve, reject) => {
            // @ts-ignore
            server.__connections.forEach((maConn) => attemptClose(maConn));
            server.close(err => (err ? reject(err) : resolve()));
        });
    };
    let peerId, listeningAddr;
    listener.listen = (ma) => {
        listeningAddr = ma;
        peerId = ma.getPeerId();
        if (peerId) {
            listeningAddr = ma.decapsulateCode(constants_1.CODE_P2P);
        }
        return new Promise((resolve, reject) => {
            const options = utils_1.multiaddrToNetConfig(listeningAddr);
            server.listen(options, (err) => {
                if (err)
                    return reject(err);
                log('Listening on %s', server.address());
                resolve();
            });
        });
    };
    listener.getAddrs = () => {
        let addrs = [];
        const address = server.address();
        if (!address) {
            throw new Error('Listener is not ready yet');
        }
        // Because TCP will only return the IPv6 version
        // we need to capture from the passed multiaddr
        if (listeningAddr.toString().startsWith('/ip4')) {
            addrs = addrs.concat(utils_1.getMultiaddrs('ip4', address.address, address.port));
        }
        else if (address.family === 'IPv6') {
            addrs = addrs.concat(utils_1.getMultiaddrs('ip6', address.address, address.port));
        }
        return addrs.map(ma => (peerId ? ma.encapsulate(`/p2p/${peerId}`) : ma));
    };
    return listener;
}
exports.createListener = createListener;
function trackConn(server, maConn) {
    // @ts-ignore
    server.__connections.push(maConn);
    const untrackConn = () => {
        // @ts-ignore
        server.__connections = server.__connections.filter(c => c !== maConn);
    };
    maConn.conn.once('close', untrackConn);
}
//# sourceMappingURL=listener.js.map