"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const net_1 = __importDefault(require("net"));
const mafmt_1 = __importDefault(require("mafmt"));
const errCode = require('err-code');
const log = require('debug')('libp2p:tcp');
const socket_to_conn_1 = require("./socket-to-conn");
const listener_1 = require("./listener");
const utils_1 = require("./utils");
const abortable_iterator_1 = require("abortable-iterator");
const constants_1 = require("./constants");
/**
 * @class TCP
 */
class TCP {
    constructor({ upgrader }) {
        if (!upgrader) {
            throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.');
        }
        this._upgrader = upgrader;
    }
    /**
     * @async
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Connection} An upgraded Connection
     */
    async dial(ma, options) {
        options = options || {};
        const socket = await this._connect(ma, options);
        const maConn = socket_to_conn_1.socketToConn(socket, { remoteAddr: ma, signal: options.signal });
        log('new outbound connection %s', maConn.remoteAddr);
        const conn = await this._upgrader.upgradeOutbound(maConn);
        log('outbound connection %s upgraded', maConn.remoteAddr);
        return conn;
    }
    /**
     * @private
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Promise<Socket>} Resolves a TCP Socket
     */
    _connect(ma, options) {
        if (options.signal && options.signal.aborted) {
            throw new abortable_iterator_1.AbortError();
        }
        return new Promise((resolve, reject) => {
            const start = Date.now();
            const cOpts = utils_1.multiaddrToNetConfig(ma);
            log('dialing %j', cOpts);
            const rawSocket = net_1.default.connect(cOpts);
            const onError = (err) => {
                err.message = `connection error ${cOpts.host}:${cOpts.port}: ${err.message}`;
                done(err);
            };
            const onTimeout = () => {
                log('connnection timeout %s:%s', cOpts.host, cOpts.port);
                const err = errCode(new Error(`connection timeout after ${Date.now() - start}ms`), 'ERR_CONNECT_TIMEOUT');
                // Note: this will result in onError() being called
                rawSocket.emit('error', err);
            };
            const onConnect = () => {
                log('connection opened %j', cOpts);
                done();
            };
            const onAbort = () => {
                log('connection aborted %j', cOpts);
                rawSocket.destroy();
                done(new abortable_iterator_1.AbortError());
            };
            const done = (err) => {
                rawSocket.removeListener('error', onError);
                rawSocket.removeListener('timeout', onTimeout);
                rawSocket.removeListener('connect', onConnect);
                options.signal && options.signal.removeEventListener('abort', onAbort);
                if (err)
                    return reject(err);
                resolve(rawSocket);
            };
            rawSocket.on('error', onError);
            rawSocket.on('timeout', onTimeout);
            rawSocket.on('connect', onConnect);
            options.signal && options.signal.addEventListener('abort', onAbort);
        });
    }
    /**
     * Creates a TCP listener. The provided `handler` function will be called
     * anytime a new incoming Connection has been successfully upgraded via
     * `upgrader.upgradeInbound`.
     * @param {*} [options]
     * @param {function(Connection)} handler
     * @returns {Listener} A TCP listener
     */
    createListener(options, handler) {
        if (typeof options === 'function') {
            handler = options;
            options = {};
        }
        options = options || {};
        return listener_1.createListener({ handler, upgrader: this._upgrader }, options);
    }
    /**
     * Takes a list of `Multiaddr`s and returns only valid TCP addresses
     * @param {Multiaddr[]} multiaddrs
     * @returns {Multiaddr[]} Valid TCP multiaddrs
     */
    filter(multiaddrs) {
        multiaddrs = Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs];
        return multiaddrs.filter(ma => {
            if (ma.protoCodes().includes(constants_1.CODE_CIRCUIT)) {
                return false;
            }
            return mafmt_1.default.TCP.matches(ma.decapsulateCode(constants_1.CODE_P2P));
        });
    }
}
exports.default = TCP;
//# sourceMappingURL=index.js.map