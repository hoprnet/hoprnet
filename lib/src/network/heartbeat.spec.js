"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
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
const heartbeat_1 = require("../interactions/network/heartbeat");
const heartbeat_2 = require("./heartbeat");
const assert_1 = __importDefault(require("assert"));
const multiaddr_1 = __importDefault(require("multiaddr"));
describe('check heartbeat mechanism', function () {
    async function generateNode() {
        const node = (await libp2p.create({
            peerInfo: await peer_info_1.default.create(await peer_id_1.default.create({ keyType: 'secp256k1' })),
            modules: {
                transport: [TCP],
                streamMuxer: [MPLEX],
                connEncryption: [SECIO]
            }
        }));
        node.peerInfo.multiaddrs.add(multiaddr_1.default('/ip4/0.0.0.0/tcp/0'));
        await node.start();
        node.peerRouting.findPeer = (_) => {
            return Promise.reject(Error('not implemented'));
        };
        node.interactions = {
            network: {
                heartbeat: new heartbeat_1.Heartbeat(node)
            }
        };
        node.network = {
            heartbeat: new heartbeat_2.Heartbeat(node)
        };
        node.log = debug_1.default(`${chalk_1.default.blue(node.peerInfo.id.toB58String())}: `);
        return node;
    }
    it('should initialise the heartbeat module and start the heartbeat functionality', async function () {
        const [Alice, Bob, Chris, Dave] = await Promise.all([generateNode(), generateNode(), generateNode(), generateNode()]);
        // Check whether our event listener is triggered by heartbeat interactions
        await Promise.all([
            new Promise(async (resolve) => {
                Bob.network.heartbeat.once('beat', (peerId) => {
                    assert_1.default(Alice.peerInfo.id.isEqual(peerId), `Incoming connection must come from Alice`);
                    resolve();
                });
            }),
            Alice.interactions.network.heartbeat.interact(Bob.peerInfo)
        ]);
        // Check whether our event listener is triggered by `normal` interactions
        await Promise.all([
            new Promise(async (resolve) => {
                Chris.network.heartbeat.once('beat', (peerId) => {
                    assert_1.default(Alice.peerInfo.id.isEqual(peerId), `Incoming connection must come from Alice`);
                    resolve();
                });
            }),
            Alice.dial(Chris.peerInfo)
        ]);
        // Check that the internal state is as expected
        assert_1.default(Alice.network.heartbeat.heap.includes(Chris.peerInfo.id.toB58String()), `Alice should know about Chris now.`);
        assert_1.default(Alice.network.heartbeat.heap.includes(Bob.peerInfo.id.toB58String()), `Alice should know about Bob now.`);
        assert_1.default(Chris.network.heartbeat.heap.includes(Alice.peerInfo.id.toB58String()), `Chris should know about Alice now.`);
        assert_1.default(Bob.network.heartbeat.heap.includes(Alice.peerInfo.id.toB58String()), `Bob should know about Alice now.`);
        // Simulate a node failure
        await Chris.stop();
        // Reset lastSeen times
        for (const peerId of Alice.network.heartbeat.nodes.keys()) {
            Alice.network.heartbeat.nodes.set(peerId, 0);
        }
        // Check whether a node failure gets detected
        await Alice.network.heartbeat.checkNodes();
        assert_1.default(!Alice.network.heartbeat.nodes.has(Chris.peerInfo.id.toB58String()), `Alice should have removed Chris.`);
        await Promise.all([
            Alice.stop(),
            Bob.stop()
        ]);
    });
});
