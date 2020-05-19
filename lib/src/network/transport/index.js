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
const abortable_iterator_1 = __importDefault(require("abortable-iterator"));
const abort_controller_1 = __importDefault(require("abort-controller"));
const listener_1 = require("./listener");
const utils_1 = require("./utils");
const abortable_iterator_2 = require("abortable-iterator");
const constants_1 = require("./constants");
const multiaddr_1 = __importDefault(require("multiaddr"));
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const it_pipe_1 = __importDefault(require("it-pipe"));
const it_pushable_1 = __importDefault(require("it-pushable"));
const simple_peer_1 = __importDefault(require("simple-peer"));
// @ts-ignore
const wrtc = require("wrtc");
const utils_2 = require("../../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const chalk_1 = __importDefault(require("chalk"));
const RELAY_REGISTER = '/hopr/relay-register/0.0.1';
const RELAY_UNREGISTER = '/hopr/relay-unregister/0.0.1';
const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1';
const DELIVERY_UNREGISTER = '/hopr/delivery-unregister/0.0.1';
const WEBRTC = '/hopr/webrtc/0.0.1';
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
    constructor({ upgrader, libp2p, bootstrapServers, }) {
        if (!upgrader) {
            throw new Error('An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.');
        }
        if (!libp2p) {
            throw new Error('Transport module needs access to libp2p.');
        }
        if (bootstrapServers !== undefined && bootstrapServers.length > 0) {
            this.relays = bootstrapServers.filter((peerInfo) => peerInfo !== undefined && !libp2p.peerInfo.id.isEqual(peerInfo.id));
            this.stunServers = [];
            for (let i = 0; i < this.relays.length; i++) {
                let urls = '';
                this.relays[i].multiaddrs.forEach((ma) => {
                    if (urls.length > 0) {
                        urls += ', ';
                    }
                    const opts = ma.toOptions();
                    if (opts.family == 'ipv4') {
                        urls += `stun:${opts.host}`;
                    }
                    else if (opts.family == 'ipv6') {
                        // WebRTC seems to have no support IPv6 addresses
                        // urls += `stun:[0${opts.host}]`
                    }
                });
                this.stunServers.push({ urls });
            }
        }
        this._registrar = libp2p.registrar;
        this._handle = libp2p.handle.bind(libp2p);
        this._unhandle = libp2p.unhandle.bind(libp2p);
        this._dialer = libp2p.dialer;
        this._peerInfo = libp2p.peerInfo;
        this._upgrader = upgrader;
        this._encoder = new TextEncoder();
        this._decoder = new TextDecoder();
        this._handle(RELAY_REGISTER, this.handleRelayRegister.bind(this));
        this._handle(RELAY_UNREGISTER, this.handleRelayUnregister.bind(this));
        this._handle(DELIVERY_REGISTER, this.handleDeliveryRegister.bind(this));
        this._handle(DELIVERY_UNREGISTER, this.handleDeliveryUnregister.bind(this));
        this._handle(WEBRTC, this.handleWebRTC.bind(this));
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
                await this.closeConnection(options.counterparty, options.relay);
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
                relay: connection.remotePeer,
            }));
            if (this.connHandler !== undefined) {
                return this.connHandler(conn);
            }
        };
    }
    forwardHandlerFactory(counterparty) {
        return (async ({ stream, connection }) => {
            let conn = this._registrar.getConnection(new peer_info_1.default(counterparty));
            if (!conn) {
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
            }
            const { stream: innerStream } = await conn.newStream([RELAY_DELIVER(connection.remotePeer.pubKey.marshal())]);
            it_pipe_1.default(stream, innerStream, stream);
        }).bind(this);
    }
    handleDeliveryUnregister({ stream }) {
        it_pipe_1.default(stream, async (source) => {
            for await (const msg of source) {
                let counterparty;
                try {
                    counterparty = await utils_2.pubKeyToPeerId(msg.slice());
                }
                catch {
                    return;
                }
                this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()));
            }
        });
    }
    handleDeliveryRegister({ stream }) {
        it_pipe_1.default(stream, (source) => {
            return async function* () {
                for await (const msg of source) {
                    let sender;
                    try {
                        sender = await utils_2.pubKeyToPeerId(msg.slice());
                    }
                    catch {
                        return yield FAIL;
                    }
                    this._handle(RELAY_DELIVER(sender.pubKey.marshal()), this.deliveryHandlerFactory(sender));
                    return yield OK;
                }
            }.apply(this);
        }, stream);
    }
    handleRelayUnregister({ stream, connection }) {
        it_pipe_1.default(
        /* prettier-ignore */
        stream, async (source) => {
            for await (const msg of source) {
                let counterparty;
                try {
                    counterparty = await utils_2.pubKeyToPeerId(msg.slice());
                }
                catch {
                    return;
                }
                this._unhandle(RELAY_FORWARD(
                /* prettier-ignore */
                connection.remotePeer.pubKey.marshal(), counterparty.pubKey.marshal()));
                let conn = this._registrar.getConnection(new peer_info_1.default(counterparty));
                if (!conn) {
                    try {
                        conn = await this._dialer.connectToPeer(new peer_info_1.default(counterparty));
                    }
                    catch (err) {
                        return;
                    }
                }
                const { stream: unRegisterStream } = await conn.newStream([DELIVERY_UNREGISTER]);
                it_pipe_1.default(
                /* prettier-ignore */
                [counterparty.pubKey.marshal()], unRegisterStream);
            }
        });
    }
    async closeConnection(counterparty, relay) {
        this._unhandle(RELAY_DELIVER(counterparty.pubKey.marshal()));
        // @TODO unregister at correct relay node
        let conn = this._registrar.getConnection(new peer_info_1.default(relay));
        if (!conn) {
            try {
                conn = await this._dialer.connectToPeer(new peer_info_1.default(relay));
            }
            catch (err) {
                console.log(`Could not request relayer ${relay.toB58String()} to tear down relayed connection. Error was:\n`, err);
                return;
            }
        }
        const { stream: unRegisterStream } = await conn.newStream([RELAY_UNREGISTER]);
        await it_pipe_1.default(
        /* prettier-ignore */
        [counterparty.pubKey.marshal()], unRegisterStream);
        return;
    }
    async registerDelivery(outerconnection, counterparty) {
        let conn = this._registrar.getConnection(new peer_info_1.default(counterparty));
        const abort = new abort_controller_1.default();
        const timeout = setTimeout(() => {
            abort.abort();
        }, constants_1.RELAY_CIRCUIT_TIMEOUT);
        if (!conn) {
            try {
                conn = await this._dialer.connectToPeer(new peer_info_1.default(counterparty), { signal: abort.signal });
            }
            catch (err) {
                console.log(`[Relayer] Could not establish relayed connection to destination ${counterparty.toB58String()}. Err was:\n`, err);
                return;
            }
        }
        const { stream: deliverRegisterStream } = await conn.newStream([DELIVERY_REGISTER]);
        let answer = await it_pipe_1.default(
        /* prettier-ignore */
        [outerconnection.remotePeer.pubKey.marshal()], deliverRegisterStream, async (source) => {
            for await (const msg of source) {
                return msg.slice();
            }
        });
        clearTimeout(timeout);
        return answer || FAIL;
    }
    handleRelayRegister({ stream, connection }) {
        it_pipe_1.default(
        /* prettier-ignore */
        stream, (source) => {
            return async function* () {
                for await (const msg of source) {
                    let counterparty;
                    try {
                        counterparty = await utils_2.pubKeyToPeerId(msg.slice());
                    }
                    catch {
                        return yield FAIL;
                    }
                    const answer = await this.registerDelivery(connection, counterparty);
                    if (hopr_utils_1.u8aEquals(answer, OK)) {
                        this._handle(RELAY_FORWARD(
                        /* prettier-ignore */
                        connection.remotePeer.pubKey.marshal(), counterparty.pubKey.marshal()), this.forwardHandlerFactory(counterparty));
                        return yield OK;
                    }
                    if (!hopr_utils_1.u8aEquals(answer, FAIL)) {
                        console.log(`Received unexpected message from counterparty '${answer}'`);
                    }
                    return yield FAIL;
                }
            }.apply(this);
        }, stream);
    }
    handleWebRTC({ stream }) {
        const queue = it_pushable_1.default();
        let channel;
        if (constants_1.USE_OWN_STUN_SERVERS) {
            channel = new simple_peer_1.default({ wrtc, trickle: true, config: { iceServers: this.stunServers } });
        }
        else {
            channel = new simple_peer_1.default({ wrtc, trickle: true });
        }
        const done = (err, conn) => {
            channel.removeListener('connect', onConnect);
            channel.removeListener('error', onError);
            channel.removeListener('signal', onSignal);
            if (err) {
                console.log(`WebRTC connection failed`);
            }
            else if (this.connHandler) {
                this.connHandler(conn);
            }
        };
        const onSignal = (msg) => {
            queue.push(this._encoder.encode(JSON.stringify(msg)));
        };
        const onConnect = async () => {
            done(undefined, await this._upgrader.upgradeInbound(socket_to_conn_1.socketToConn(channel)));
        };
        const onError = (err) => {
            done(err);
        };
        channel.on('signal', onSignal);
        channel.once('connect', onConnect);
        channel.once('error', onConnect);
        it_pipe_1.default(queue, stream, async (source) => {
            for await (const msg of source) {
                channel.signal(this._decoder.decode(msg.slice()));
            }
        });
    }
    /**
     * @async
     * @param {Multiaddr} ma
     * @param {object} options
     * @param {AbortSignal} options.signal Used to abort dial requests
     * @returns {Connection} An upgraded Connection
     */
    async dial(ma, options) {
        var _a;
        options = options || {};
        try {
            return await this.dialDirectly(ma, options);
        }
        catch (err) {
            const destination = peer_id_1.default.createFromCID(ma.getPeerId());
            if (this.relays === undefined) {
                throw Error(`Could not connect ${chalk_1.default.yellow(ma.toString())} because there was no relay defined. Connection error was:\n${err}`);
            }
            // Check whether we know some relays that we can use
            const potentialRelays = (_a = this.relays) === null || _a === void 0 ? void 0 : _a.filter((peerInfo) => !peerInfo.id.isEqual(destination));
            if (potentialRelays == null || potentialRelays.length == 0) {
                throw Error(`Destination ${chalk_1.default.yellow(ma.toString())} cannot be accessed and directly and there is no other relay node known. Connection error was:\n${err}`);
            }
            return await this.dialWithRelay(ma, potentialRelays, options);
        }
    }
    tryWebRTC(conn, counterparty, options) {
        return new Promise(async (resolve, reject) => {
            const { stream } = await conn.newStream([WEBRTC]);
            const queue = it_pushable_1.default();
            let channel;
            if (constants_1.USE_OWN_STUN_SERVERS) {
                channel = new simple_peer_1.default({
                    wrtc,
                    initiator: true,
                    trickle: true,
                    config: { iceServers: this.stunServers },
                });
            }
            else {
                channel = new simple_peer_1.default({
                    wrtc,
                    initiator: true,
                    trickle: true,
                });
            }
            const done = (err, conn) => {
                channel.removeListener('connect', onConnect);
                channel.removeListener('error', onError);
                channel.removeListener('signal', onSignal);
                options.signal && options.signal.removeEventListener('abort', onAbort);
                if (err) {
                    reject(err);
                }
                else {
                    resolve(conn);
                }
            };
            const onAbort = () => {
                channel.destroy();
                setImmediate(reject);
            };
            const onSignal = (data) => {
                queue.push(this._encoder.encode(JSON.stringify(data)));
            };
            const onConnect = async () => {
                done(undefined, await this._upgrader.upgradeOutbound(socket_to_conn_1.socketToConn(channel)));
            };
            const onError = (err) => {
                done(err);
            };
            channel.on('signal', onSignal);
            channel.once('error', onError);
            channel.once('connect', onConnect);
            it_pipe_1.default(
            /* prettier-ignore */
            queue, stream, async (source) => {
                for await (const msg of source) {
                    channel.signal(this._decoder.decode(msg.slice()));
                }
            });
        });
    }
    async dialWithRelay(ma, relays, options) {
        const destination = peer_id_1.default.createFromCID(ma.getPeerId());
        let relayConnection = await Promise.race(relays.map((relay) => new Promise(async (resolve) => {
            log(`[${chalk_1.default.blue(this._peerInfo.id.toB58String())}] trying to call ${chalk_1.default.yellow(ma.toString())} over relay node ${chalk_1.default.yellow(relay.id.toB58String())}`);
            let relayConnection = this._registrar.getConnection(relay);
            if (!relayConnection) {
                try {
                    return resolve(await this._dialer.connectToPeer(relay, { signal: options === null || options === void 0 ? void 0 : options.signal }));
                }
                catch { }
            }
        })));
        if (!relayConnection) {
            throw Error(`Unable to establish a connection to any known relay node. Tried ${chalk_1.default.yellow(relays.map((relay) => relay.id.toB58String()).join(`, `))}`);
        }
        const { stream: registerStream } = await relayConnection.newStream([RELAY_REGISTER]);
        const answer = await it_pipe_1.default(
        /* prettier-ignore */
        [destination.pubKey.marshal()], registerStream, async (source) => {
            for await (const msg of source) {
                return msg.slice();
            }
        });
        if (!hopr_utils_1.u8aEquals(answer, OK)) {
            throw Error(`Register relaying failed. Received '${this._decoder.decode(answer)}'.`);
        }
        const { stream: msgStream } = await relayConnection.newStream([
            RELAY_FORWARD(this._peerInfo.id.pubKey.marshal(), destination.pubKey.marshal()),
        ]);
        if (options.signal) {
            msgStream.source = abortable_iterator_1.default(msgStream.source, options.signal);
        }
        let conn = await this._upgrader.upgradeOutbound(this.relayToConn({
            stream: msgStream,
            counterparty: destination,
            relay: relayConnection.remotePeer,
        }));
        try {
            let webRTCConn = await this.tryWebRTC(conn, destination, { signal: options.signal });
            conn.close();
            return webRTCConn;
        }
        catch (err) {
            console.log(err);
        }
        return conn;
    }
    async dialDirectly(ma, options) {
        log(`[${chalk_1.default.blue(this._peerInfo.id.toB58String())}] dailing ${chalk_1.default.yellow(ma.toString())} directly`);
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
            throw new abortable_iterator_2.AbortError();
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
                done(new abortable_iterator_2.AbortError());
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