"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getPeerInfo = void 0;
const libp2p_crypto_1 = require("libp2p-crypto");
const chalk_1 = __importDefault(require("chalk"));
const __1 = require("..");
const hopr_demo_seeds_1 = require("@hoprnet/hopr-demo-seeds");
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const multiaddr_1 = __importDefault(require("multiaddr"));
const dbKeys_1 = require("../../dbKeys");
const constants_1 = require("../../constants");
/**
 * Assemble the addresses that we are using
 */
function getAddrs(options) {
    const addrs = [];
    if (options.hosts === undefined || (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined)) {
        let ip4Port = 9091;
        // ============================= Only for testing ============================================================
        if (options.id != null && Number.isInteger(options.id)) {
            ip4Port += (options.id + 1) * 2;
        }
        // ===========================================================================================================
        addrs.push(multiaddr_1.default(`/ip4/0.0.0.0/tcp/${ip4Port}`));
    }
    if (options.hosts !== undefined) {
        if (options.hosts.ip4 === undefined && options.hosts.ip6 === undefined) {
            throw Error(`Unable `);
        }
        if (options.hosts.ip4 !== undefined) {
            let ip4Port = options.hosts.ip4.port;
            // ============================= Only for testing ============================================================
            if (options.id != null && Number.isInteger(options.id)) {
                ip4Port += (options.id + 1) * 2;
            }
            // ===========================================================================================================
            addrs.push(multiaddr_1.default(`/ip4/${options.hosts.ip4.ip}/tcp/${ip4Port}`));
        }
        if (options.hosts.ip6 !== undefined) {
            let ip6Port = options.hosts.ip6.port;
            // ============================= Only for testing ============================================================
            if (options.id != null && Number.isInteger(options.id)) {
                ip6Port += (options.id + 1) * 2;
            }
            // ===========================================================================================================
            addrs.push(multiaddr_1.default(`/ip6/${options.hosts.ip6.ip}/tcp/${ip6Port}`));
        }
    }
    return addrs;
}
/**
 * Checks whether we have gotten any peerId in through the options.
 */
async function getPeerId(options, db) {
    if (options.peerId != null && peer_id_1.default.isPeerId(options.peerId)) {
        return options.peerId;
    }
    if (options.debug) {
        if (options.id != null && isFinite(options.id)) {
            if (options.bootstrapNode) {
                if (options.id >= hopr_demo_seeds_1.BOOTSTRAP_SEEDS.length) {
                    throw Error(`Unable to access bootstrap seed number ${options.id} out of ${hopr_demo_seeds_1.BOOTSTRAP_SEEDS.length} bootstrap seeds.`);
                }
                return await __1.privKeyToPeerId(hopr_demo_seeds_1.BOOTSTRAP_SEEDS[options.id]);
            }
            else {
                if (options.id >= hopr_demo_seeds_1.NODE_SEEDS.length) {
                    throw Error(`Unable to access node seed number ${options.id} out of ${hopr_demo_seeds_1.NODE_SEEDS.length} node seeds.`);
                }
                return await __1.privKeyToPeerId(hopr_demo_seeds_1.NODE_SEEDS[options.id]);
            }
        }
        else if (options.bootstrapNode) {
            return await __1.privKeyToPeerId(hopr_demo_seeds_1.BOOTSTRAP_SEEDS[0]);
        }
    }
    else if (options.id != null && isFinite(options.id)) {
        throw Error(`Demo Ids are only available in DEBUG_MODE. Consider setting DEBUG_MODE to 'true' in .env`);
    }
    if (db == null) {
        throw Error('Cannot get/store any peerId without a database handle.');
    }
    return getFromDatabase(db, options.password);
}
/**
 * Try to retrieve Id from database
 */
async function getFromDatabase(db, pw) {
    let serializedKeyPair;
    try {
        serializedKeyPair = await db.get(Buffer.from(dbKeys_1.KeyPair));
    }
    catch (err) {
        return createIdentity(db, pw);
    }
    return recoverIdentity(serializedKeyPair, pw);
}
async function recoverIdentity(serializedKeyPair, pw) {
    let peerId;
    let done = false;
    if (pw !== undefined) {
        try {
            peerId = await __1.deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw));
            done = true;
        }
        catch (err) {
            console.log(`Could not recover id from database with given password. Please type it in manually.`);
        }
    }
    while (!done) {
        pw = await __1.askForPassword('Please type in the passwort that was used to encrypt to key.');
        try {
            peerId = await __1.deserializeKeyPair(serializedKeyPair, new TextEncoder().encode(pw));
            done = true;
        }
        catch { }
    }
    console.log(`Successfully recovered ${chalk_1.default.blue(peerId.toB58String())} from database.`);
    return peerId;
}
async function createIdentity(db, pw) {
    pw = pw || (await __1.askForPassword('Please type in a password to encrypt the secret key.'));
    const key = await libp2p_crypto_1.keys.generateKeyPair('secp256k1', 256);
    const peerId = await peer_id_1.default.createFromPrivKey(key.bytes);
    const serializedKeyPair = await __1.serializeKeyPair(peerId, new TextEncoder().encode(pw));
    await db.put(Buffer.from(dbKeys_1.KeyPair), serializedKeyPair);
    return peerId;
}
async function getPeerInfo(options, db) {
    if (db == null && (options == null || (options != null && options.peerInfo == null && options.peerId == null))) {
        throw Error('Invalid input parameter. Please set a valid peerInfo or pass a database handle.');
    }
    const addrs = getAddrs(options);
    let peerInfo;
    if (options.peerInfo != null) {
        peerInfo = options.peerInfo;
    }
    else {
        peerInfo = new peer_info_1.default(await getPeerId(options, db));
    }
    addrs.forEach(addr => peerInfo.multiaddrs.add(addr.encapsulate(`/${constants_1.NAME}/${peerInfo.id.toB58String()}`)));
    return peerInfo;
}
exports.getPeerInfo = getPeerInfo;
//# sourceMappingURL=getPeerInfo.js.map