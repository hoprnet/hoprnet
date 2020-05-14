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
const multiaddr_1 = __importDefault(require("multiaddr"));
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const utils_2 = require("../../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const RELAY_REGISTER = '/hopr/relay-register/0.0.1';
const RELAY_UNREGISTER = '/hopr/relay-unregister/0.0.1';
const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1';
const RELAY_DELIVER = (from) => `/hopr/deliver${hopr_utils_1.u8aToHex(from)}/0.0.1`;
const RELAY_FORWARD = (from, to) => {
    if (from.length !== to.length) {
        throw Error(`Could not generate RELAY_FORWARD protocol string because array lengths do not match`);
    }
    return `/hopr/forward${hopr_utils_1.u8aToHex(hopr_utils_1.u8aAdd(false, from, to))}/0.0.1`;
};
const OK = new TextEncoder().encode('OK');
const FAIL = new TextEncoder().encode('FAIL');
/**
 * @class TCP
 */
class TCP {
    constructor({ upgrader, libp2p, bootstrap }) {
        if (!upgrader) {
            throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.');
        }
        if (!libp2p) {
            throw new Error('Transport module needs access to libp2p.');
        }
        this.relay = bootstrap;
        this._registrar = libp2p.registrar;
        this._handle = libp2p.handle.bind(libp2p);
        this._unhandle = libp2p.unhandle.bind(libp2p);
        this._dialer = libp2p.dialer;
        this._peerInfo = libp2p.peerInfo;
        this._upgrader = upgrader;
        this._handle(RELAY_REGISTER, this.handleRelayRegister.bind(this));
        this._handle(RELAY_UNREGISTER, this.handleRelayUnregister.bind(this));
        this._handle(DELIVERY_REGISTER, this.handleDeliveryRegister.bind(this));
    }
    get [Symbol.toStringTag]() {
        return 'TCP';
    }
    relayToConn(options) {
        const maConn = {
            ...options.stream,
            conn: options.stream,
            remoteAddr: multiaddr_1.default(`/p2p/${options.counterparty.toB58String()}`),
            close: async (err) => {
                if (err !== undefined) {
                    console.log(err);
                }
                await this.closeConnection(options.counterparty);
                maConn.timeline.close = Date.now();
            },
            timeline: {
                open: Date.now(),
            },
        };
        return maConn;
    }
    deliveryHandlerFactory(sender) {
        return async ({ stream, connection }) => {
            const conn = await this._upgrader.upgradeInbound(this.relayToConn({
                stream,
                counterparty: sender,
            }));
            if (this.connHandler !== undefined) {
                return this.connHandler(conn);
            }
        };
    }
    forwardHandlerFactory(counterparty) {
        return (async ({ stream, connection }) => {
            let conn;
            try {
                conn = await this._dialer.connectToPeer(new peer_info_1.default(counterparty));
            }
            catch (err) {
                console.log(`Could not forward packet to ${counterparty.toB58String()}. Error was :\n`, err);
                try {
                    it_pipe_1.default([FAIL], stream);
                }
                catch (err) {
                    console.log(`Failed to inform counterparty ${connection.remotePeer.toB58String()}`);
                }
                return;
            }
            const { stream: innerStream } = await conn.newStream([RELAY_DELIVER(connection.remotePeer.pubKey.marshal())]);
            it_pipe_1.default(stream, innerStream, stream);
        }).bind(this);
    }
    handleDeliveryRegister({ stream }) {
        it_pipe_1.default(stream, (source) => {
            return async function* () {
                for await (const msg of source) {
                    const sender = await utils_2.pubKeyToPeerId(msg.slice());
                    this._handle(RELAY_DELIVER(sender.pubKey.marshal()), this.deliveryHandlerFactory(sender));
                    yield OK;
                }
            }.apply(this);
        }, stream);
    }
    handleRelayUnregister({ stream, connection }) {
        it_pipe_1.default(
        /* prettier-ignore */
        stream, (source) => {
            return async function* () {
                for await (const msg of source) {
                    const counterparty = await utils_2.pubKeyToPeerId(msg.slice());
                    try {
                        this._unhandle(RELAY_FORWARD(
                        /* prettier-ignore */
                        connection.remotePeer.pubKey.marshal(), counterparty.pubKey.marshal()));
                    }
                    catch (err) {
                        console.log(err);
                    }
                    let conn;
                    try {
                        conn = await this._dialer.connectToPeer(new peer_info_1.default(counterparty));
                    }
                    catch (err) { }
                    yield OK;
                }
            }.apply(this);
        }, stream);
    }
    async closeConnection(counterparty) {
        this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()));
        let conn;
        try {
            conn = await this._dialer.connectToPeer(this.relay);
        }
        catch (err) {
            console.log(`Could not request relayer ${this.relay.id.toB58String()} to tear down relayed connection. Error was:\n`, err);
            return;
        }
        const { stream } = await conn.newStream([RELAY_UNREGISTER]);
        await it_pipe_1.default(
        /* prettier-ignore */
        [counterparty.pubKey.marshal()], stream, async (source) => {
            for await (const msg of source) {
                return msg.slice();
            }
        });
        return;
    }
    async registerDelivery(outerconnection, counterparty) {
        let conn;
        try {
            conn = await this._dialer.connectToPeer(new peer_info_1.default(counterparty));
        }
        catch (err) {
            console.log(err);
            return;
        }
        const { stream } = await conn.newStream([DELIVERY_REGISTER]);
        return await it_pipe_1.default(
        /* prettier-ignore */
        [outerconnection.remotePeer.pubKey.marshal()], stream, async (source) => {
            for await (const msg of source) {
                return msg.slice();
            }
        });
    }
    handleRelayRegister({ stream, connection }) {
        it_pipe_1.default(
        /* prettier-ignore */
        stream, (source) => {
            return async function* () {
                for await (const msg of source) {
                    const counterparty = await utils_2.pubKeyToPeerId(msg.slice());
                    // setImmediate
                    // make this non-blocking
                    const answer = await this.registerDelivery(connection, counterparty);
                    if (hopr_utils_1.u8aEquals(answer, OK)) {
                        this._handle(RELAY_FORWARD(
                        /* prettier-ignore */
                        connection.remotePeer.pubKey.marshal(), counterparty.pubKey.marshal()), this.forwardHandlerFactory(counterparty));
                        yield OK;
                    }
                    else if (hopr_utils_1.u8aEquals(answer, FAIL)) {
                        yield FAIL;
                    }
                    else {
                        console.log(`Received unexpected message from counterparty '${answer}'`);
                    }
                }
            }.apply(this);
        }, stream);
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
        try {
            return await this.dialDirectly(ma, options);
        }
        catch (err) {
            if (this.relay === undefined) {
                throw err;
            }
            return await this.dialWithRelay(ma, options);
        }
    }
    async dialWithRelay(ma, options) {
        const destinationPeerId = peer_id_1.default.createFromCID(ma.getPeerId());
        console.log(`dailing ${ma.toString()} over relay node`);
        let relayConnection = this._registrar.getConnection(this.relay);
        if (!relayConnection) {
            relayConnection = await this._dialer.connectToPeer(this.relay);
        }
        const { stream: registerStream } = await relayConnection.newStream([RELAY_REGISTER]);
        const answer = await it_pipe_1.default(
        /* prettier-ignore */
        [destinationPeerId.pubKey.marshal()], registerStream, async (source) => {
            for await (const msg of source) {
                return msg.slice();
            }
        });
        if (!hopr_utils_1.u8aEquals(answer, OK)) {
            throw Error(`Register relaying failed. Received '${answer}'.`);
        }
        const { stream: msgStream } = await relayConnection.newStream([
            RELAY_FORWARD(this._peerInfo.id.pubKey.marshal(), destinationPeerId.pubKey.marshal()),
        ]);
        return await this._upgrader.upgradeOutbound(this.relayToConn({
            stream: msgStream,
            counterparty: destinationPeerId,
        }));
    }
    async dialDirectly(ma, options) {
        console.log(`dailing ${ma.toString()} directly`);
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
            console.log('dialing %j', cOpts);
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
        this.connHandler = handler;
        return listener_1.createListener({ handler, upgrader: this._upgrader }, options);
    }
    /**
     * Takes a list of `Multiaddr`s and returns only valid TCP addresses
     * @param multiaddrs
     * @returns Valid TCP multiaddrs
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