"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
// @ts-ignore
const libp2p = require("libp2p");
// @ts-ignore
const MPLEX = require("libp2p-mplex");
// @ts-ignore
const KadDHT = require("libp2p-kad-dht");
// @ts-ignore
const SECIO = require("libp2p-secio");
// import { WebRTCv4, WebRTCv6 } = require('./network/natTraversal')
// @ts-ignore
const TCP = require("libp2p-tcp");
// @ts-ignore
const defaultsDeep = require("@nodeutils/defaults-deep");
const packet_1 = require("./messages/packet");
const constants_1 = require("./constants");
const network_1 = require("./network");
const utils_1 = require("./utils");
const levelup_1 = __importDefault(require("levelup"));
const leveldown_1 = __importDefault(require("leveldown"));
const chalk_1 = __importDefault(require("chalk"));
const debug_1 = __importDefault(require("debug"));
const peer_id_1 = __importDefault(require("peer-id"));
const interactions_1 = require("./interactions");
const DbKeys = __importStar(require("./db_keys"));
class Hopr extends libp2p {
    /**
     * @constructor
     *
     * @param _options
     * @param provider
     */
    constructor(options, db, paymentChannels) {
        super(defaultsDeep({
            peerInfo: options.peerInfo
        }, {
            // Disable libp2p-switch protections for the moment
            switch: {
                denyTTL: 1,
                denyAttempts: Infinity
            },
            // The libp2p modules for this libp2p bundle
            modules: {
                transport: [
                    TCP
                    // WebRTCv4,
                    // WebRTCv6
                ],
                streamMuxer: [MPLEX],
                connEncryption: [SECIO],
                dht: KadDHT
                // peerDiscovery: [
                //     WebRTC.discovery
                // ]
            },
            config: {
                // peerDiscovery: {
                //     webRTCStar: {
                //         enabled: true
                //     }
                // },
                dht: {
                    enabled: true
                },
                relay: {
                    enabled: false
                }
            }
        }));
        this.db = db;
        this.paymentChannels = paymentChannels;
        this.dbKeys = DbKeys;
        this.output = options.output;
        this.bootstrapServers = options.bootstrapServers;
        this.isBootstrapNode = options.bootstrapNode;
        this.interactions = new interactions_1.Interactions(this);
        this.network = new network_1.Network(this);
        this.log = debug_1.default(`${chalk_1.default.blue(this.peerInfo.id.toB58String())}: `);
    }
    /**
     * Creates a new node.
     *
     * @param options the parameters
     */
    static async createNode(options) {
        const db = Hopr.openDatabase(`db`, options.connector.constants, options);
        if (options.peerInfo == null) {
            options.peerInfo = await utils_1.getPeerInfo(options, db);
        }
        let connector = await options.connector.create(db, options.peerInfo.id.privKey.marshal(), {
            id: options.id,
            provider: options.provider
        });
        return new Hopr(options, db, connector).up();
    }
    /**
     * Parses the bootstrap servers given in `.env` and tries to connect to each of them.
     *
     * @throws an error if none of the bootstrapservers is online
     */
    async connectToBootstrapServers() {
        const results = await Promise.all(this.bootstrapServers.map(addr => this.dial(addr).then(() => true, () => false)));
        if (!results.some(online => online)) {
            throw Error('Unable to connect to any bootstrap server.');
        }
    }
    /**
     * This method starts the node and registers all necessary handlers. It will
     * also open the database and creates one if it doesn't exists.
     *
     * @param options
     */
    async up() {
        var _a, _b;
        await super.start();
        if (!this.isBootstrapNode) {
            await this.connectToBootstrapServers();
        }
        else {
            this.log(`Available under the following addresses:`);
            this.peerInfo.multiaddrs.forEach((ma) => {
                this.log(ma.toString());
            });
        }
        await ((_a = this.paymentChannels) === null || _a === void 0 ? void 0 : _a.start());
        (_b = this.network.heartbeat) === null || _b === void 0 ? void 0 : _b.start();
        return this;
    }
    /**
     * Shuts down the node and saves keys and peerBook in the database
     */
    async down() {
        var _a, _b, _c;
        await ((_a = this.db) === null || _a === void 0 ? void 0 : _a.close());
        this.log(`Database closed.`);
        (_b = this.network.heartbeat) === null || _b === void 0 ? void 0 : _b.stop();
        await ((_c = this.paymentChannels) === null || _c === void 0 ? void 0 : _c.stop());
        this.log(`Connector stopped.`);
        await super.stop();
    }
    /**
     * Sends a message.
     *
     * @notice THIS METHOD WILL SPEND YOUR ETHER.
     * @notice This method will fail if there are not enough funds to open
     * the required payment channels. Please make sure that there are enough
     * funds controlled by the given key pair.
     *
     * @param msg message to send
     * @param destination PeerId of the destination
     * @param intermediateNodes optional set path manually
     * the acknowledgement of the first hop
     */
    async sendMessage(msg, destination, getIntermediateNodesManually) {
        if (!destination)
            throw Error(`Expecting a non-empty destination.`);
        const promises = [];
        for (let n = 0; n < msg.length / constants_1.PACKET_SIZE; n++) {
            promises.push(new Promise(async (resolve, reject) => {
                let path;
                if (getIntermediateNodesManually != undefined) {
                    path = await getIntermediateNodesManually();
                }
                else {
                    path = await this.getIntermediateNodes(destination);
                }
                path.push(destination);
                let packet;
                try {
                    packet = await packet_1.Packet.create(
                    /* prettier-ignore */
                    this, msg.slice(n * constants_1.PACKET_SIZE, Math.min(msg.length, (n + 1) * constants_1.PACKET_SIZE)), await Promise.all(path.map(utils_1.addPubKey)));
                }
                catch (err) {
                    return reject(err);
                }
                this.interactions.packet.acknowledgment.once(utils_1.u8aToHex(this.dbKeys.UnAcknowledgedTickets(path[0].pubKey.marshal(), packet.ticket.ticket.challenge)), () => {
                    console.log(`received acknowledgement`);
                    resolve();
                });
                try {
                    await this.interactions.packet.forward.interact(path[0], packet);
                }
                catch (err) {
                    return reject(err);
                }
            }));
        }
        try {
            await Promise.all(promises);
        }
        catch (err) {
            this.log(`Could not send message. Error was: ${chalk_1.default.red(err.message)}`);
        }
    }
    /**
     * Ping a node.
     *
     * @param destination PeerId of the node
     * @returns latency
     */
    async ping(destination) {
        if (!peer_id_1.default.isPeerId(destination)) {
            throw Error(`Expecting a non-empty destination.`);
        }
        const latency = await super.ping(destination);
        if (typeof latency === 'undefined') {
            throw Error('node unreachable');
        }
        return latency;
    }
    /**
     * Takes a destination and samples randomly intermediate nodes
     * that will relay that message before it reaches its destination.
     *
     * @param destination instance of peerInfo that contains the peerId of the destination
     */
    async getIntermediateNodes(destination) {
        const filter = (peerInfo) => !peerInfo.id.isEqual(this.peerInfo.id) &&
            !peerInfo.id.isEqual(destination) &&
            // exclude bootstrap server(s) from crawling results
            !this.bootstrapServers.some((pInfo) => pInfo.id.isEqual(peerInfo.id));
        await this.network.crawler.crawl(filter);
        const array = [];
        for (const peerInfo of this.peerStore.peers.values()) {
            array.push(peerInfo);
        }
        return utils_1.randomSubset(array, constants_1.MAX_HOPS - 1, filter).map((peerInfo) => peerInfo.id);
    }
    static openDatabase(db_dir, constants, options) {
        db_dir += `/${constants.CHAIN_NAME}/${constants.NETWORK}/`;
        if (options != null && options.bootstrapNode) {
            db_dir += `bootstrap`;
        }
        else if (options != null && options.id != null && Number.isInteger(options.id)) {
            // For testing ...
            db_dir += `node_${options.id}`;
        }
        else {
            db_dir += `node`;
        }
        utils_1.createDirectoryIfNotExists(db_dir);
        return levelup_1.default(leveldown_1.default(db_dir));
    }
}
exports.default = Hopr;
