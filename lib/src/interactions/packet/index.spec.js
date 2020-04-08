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
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
// @ts-ignore
const libp2p = require("libp2p");
// @ts-ignore
const TCP = require("libp2p-tcp");
// @ts-ignore
const MPLEX = require("libp2p-mplex");
// @ts-ignore
const SECIO = require("libp2p-secio");
const debug_1 = __importDefault(require("debug"));
const chalk_1 = __importDefault(require("chalk"));
const rlp_1 = require("rlp");
const packet_1 = require("../../messages/packet");
const srml_types_1 = require("@hoprnet/hopr-core-polkadot/lib/srml_types");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const keyring_1 = __importDefault(require("@polkadot/keyring"));
const types_1 = require("@polkadot/types");
const _1 = require(".");
const onChainKey_1 = require("../payments/onChainKey");
const levelup_1 = __importDefault(require("levelup"));
const memdown_1 = __importDefault(require("memdown"));
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_core_polkadot_1 = __importStar(require("@hoprnet/hopr-core-polkadot"));
const crypto_1 = require("crypto");
const DbKeys = __importStar(require("../../db_keys"));
const utils_1 = require("../../utils");
const assert_1 = __importDefault(require("assert"));
const multiaddr_1 = __importDefault(require("multiaddr"));
describe('check packet forwarding & acknowledgement generation', function () {
    const channels = new Map();
    const states = new Map();
    const typeRegistry = new types_1.TypeRegistry();
    typeRegistry.register(srml_types_1.SRMLTypes);
    async function generateNode() {
        const db = levelup_1.default(memdown_1.default());
        const node = (await libp2p.create({
            peerInfo: await peer_info_1.default.create(await peer_id_1.default.create({ keyType: 'secp256k1' })),
            modules: {
                transport: [TCP],
                streamMuxer: [MPLEX],
                connEncryption: [SECIO]
            }
        }));
        node.db = db;
        node.peerInfo.multiaddrs.add(multiaddr_1.default('/ip4/0.0.0.0/tcp/0'));
        await Promise.all([
            /* prettier-ignore */
            node.start(),
            wasm_crypto_1.waitReady()
        ]);
        node.peerRouting.findPeer = (_) => {
            return Promise.reject(Error('not implemented'));
        };
        node.interactions = {
            packet: new _1.PacketInteractions(node),
            payments: {
                onChainKey: new onChainKey_1.OnChainKey(node)
            }
        };
        const onChainKeyPair = new keyring_1.default({ type: 'sr25519' }).addFromSeed(node.peerInfo.id.pubKey.marshal().slice(0, 32), undefined, 'sr25519');
        node.paymentChannels = new hopr_core_polkadot_1.default({
            once(eventName, fn) {
                if (eventName === 'disconnected') {
                    return fn();
                }
            },
            disconnect: () => { },
            isReady: Promise.resolve(true),
            query: {
                hopr: {
                    channels(channelId) {
                        if (!channels.has(channelId.toHex())) {
                            throw Error(`missing channel ${channelId.toHex()}`);
                        }
                        return Promise.resolve(channels.get(channelId.toHex()));
                    },
                    states(accountId) {
                        if (!states.has(accountId.toHex())) {
                            throw Error(`party ${accountId.toHex()} has not set any on-chain secrets.`);
                        }
                        return Promise.resolve(states.get(accountId.toHex()));
                    }
                },
                system: {
                    events(_handler) { },
                    accountNonce() {
                        return Promise.resolve({
                            toNumber: () => 0
                        });
                    }
                }
            },
            tx: {
                hopr: {
                    init(secret, publicKey) {
                        const signAndSend = (keyPair) => {
                            states.set(new srml_types_1.AccountId(typeRegistry, keyPair.publicKey).toHex(), {
                                secret,
                                publicKey
                            });
                            return Promise.resolve();
                        };
                        return { signAndSend };
                    }
                }
            },
            createType(name, ...args) {
                const result = new (typeRegistry.get(name))(typeRegistry, ...args);
                return result;
            },
            registry: typeRegistry
        }, {
            publicKey: node.peerInfo.id.pubKey.marshal(),
            privateKey: node.peerInfo.id.privKey.marshal(),
            onChainKeyPair
        }, db);
        await node.paymentChannels.start();
        await node.paymentChannels.initOnchainValues();
        node.log = debug_1.default(`${chalk_1.default.blue(node.peerInfo.id.toB58String())}: `);
        node.dbKeys = DbKeys;
        return node;
    }
    it('should forward a packet and receive aknowledgements', async function () {
        const [Alice, Bob, Chris, Dave] = await Promise.all([generateNode(), generateNode(), generateNode(), generateNode()]);
        connectionHelper(Alice, Bob, Chris, Dave);
        const channel = hopr_core_polkadot_1.Types.Channel.createActive({
            balance: new bn_js_1.default(12345),
            balance_a: new bn_js_1.default(123)
        });
        const [channelId, channelIdSecond, channelIdThird] = await getIds(typeRegistry, Alice, Bob, Chris, Dave);
        const channelRecord = await hopr_core_polkadot_1.Types.SignedChannel.create(Bob.paymentChannels, undefined, {
            channel,
        });
        channels.set(channelIdThird.toHex(), channelRecord);
        channels.set(channelIdSecond.toHex(), channelRecord);
        channels.set(channelId.toHex(), channelRecord);
        await channelDbHelper(typeRegistry, channelRecord, Alice, Bob, Chris, Dave);
        const testMsg = crypto_1.randomBytes(utils_1.randomInteger(37, 131));
        const emitPromises = [];
        emitPromises.push(emitCheckerHelper(Alice, Bob.peerInfo.id));
        emitPromises.push(emitCheckerHelper(Bob, Chris.peerInfo.id));
        Chris.output = (arr) => {
            const [msg] = rlp_1.decode(Buffer.from(arr));
            assert_1.default(utils_1.u8aEquals(msg, testMsg), `Checks that we receive the right message.`);
        };
        await Alice.interactions.packet.forward.interact(Bob.peerInfo, await packet_1.Packet.create(Alice, rlp_1.encode([testMsg, new TextEncoder().encode(Date.now().toString())]), [Bob.peerInfo.id, Chris.peerInfo.id]));
        const testMsgSecond = crypto_1.randomBytes(utils_1.randomInteger(33, 129));
        Dave.output = (arr) => {
            const [msg] = rlp_1.decode(Buffer.from(arr));
            assert_1.default(utils_1.u8aEquals(msg, testMsgSecond), `Checks that we receive the right message.`);
        };
        emitPromises.push(emitCheckerHelper(Chris, Dave.peerInfo.id));
        await Alice.interactions.packet.forward.interact(Bob.peerInfo, await packet_1.Packet.create(Alice, rlp_1.encode([testMsgSecond, new TextEncoder().encode(Date.now().toString())]), [
            Bob.peerInfo.id,
            Chris.peerInfo.id,
            Dave.peerInfo.id
        ]));
        try {
            await Promise.all(emitPromises);
        }
        catch (err) {
            assert_1.default.fail(`Checks that we emit an event once we got an acknowledgement.`);
        }
        await Promise.all([
            Alice.paymentChannels.stop(),
            Bob.paymentChannels.stop(),
            Chris.paymentChannels.stop(),
            Dave.paymentChannels.stop()
        ]);
        await Promise.all([
            Alice.stop(),
            Bob.stop(),
            Chris.stop(),
            Dave.stop()
        ]);
    });
    // afterEach(function() {
    //   channels.clear()
    // })
});
/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper(...nodes) {
    for (let i = 0; i < nodes.length; i++) {
        for (let j = i + 1; j < nodes.length; j++) {
            nodes[i].peerStore.put(nodes[j].peerInfo);
            nodes[j].peerStore.put(nodes[i].peerInfo);
        }
    }
}
/**
 * Returns a Promise that resolves once the acknowledgement has been received
 * @param node our Hopr node
 * @param sender the sender of the packet
 */
function emitCheckerHelper(node, sender) {
    return new Promise((resolve, reject) => {
        node.interactions.packet.acknowledgment.emit = (event) => {
            node.dbKeys.UnAcknowledgedTicketsParse(utils_1.stringToU8a(event)).then(([counterparty]) => {
                if (utils_1.u8aEquals(sender.pubKey.marshal(), counterparty.pubKey.marshal())) {
                    resolve();
                }
                else {
                    reject();
                }
            }, reject);
            return false;
        };
    });
}
async function channelDbHelper(typeRegistry, record, ...nodes) {
    const promises = [];
    if (nodes.length < 2) {
        throw Error('cannot do this with less than two nodes.');
    }
    promises.push(nodes[0].db.put(Buffer.from(nodes[0].paymentChannels.dbKeys.Channel(new srml_types_1.AccountId(typeRegistry, nodes[1].paymentChannels.self.onChainKeyPair.publicKey))), Buffer.from(record)));
    for (let i = 1; i < nodes.length - 1; i++) {
        promises.push(nodes[i].db
            .batch()
            .put(Buffer.from(nodes[i].paymentChannels.dbKeys.Channel(new srml_types_1.AccountId(typeRegistry, nodes[i - 1].paymentChannels.self.onChainKeyPair.publicKey))), Buffer.from(record))
            .put(Buffer.from(nodes[i].paymentChannels.dbKeys.Channel(new srml_types_1.AccountId(typeRegistry, nodes[i + 1].paymentChannels.self.onChainKeyPair.publicKey))), Buffer.from(record))
            .write());
    }
    await Promise.all(promises);
}
function getIds(typeRegistry, ...nodes) {
    const promises = [];
    for (let i = 0; i < nodes.length - 1; i++) {
        promises.push(nodes[i].paymentChannels.utils.getId(new srml_types_1.AccountId(typeRegistry, nodes[i].paymentChannels.self.onChainKeyPair.publicKey), new srml_types_1.AccountId(typeRegistry, nodes[i + 1].paymentChannels.self.onChainKeyPair.publicKey)));
    }
    return Promise.all(promises);
}
