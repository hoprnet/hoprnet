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
const heartbeat_1 = require("./heartbeat");
const assert_1 = __importDefault(require("assert"));
const multiaddr_1 = __importDefault(require("multiaddr"));
const events_1 = require("events");
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
            heartbeat: new events_1.EventEmitter()
        };
        node.log = debug_1.default(`${chalk_1.default.blue(node.peerInfo.id.toB58String())}: `);
        return node;
    }
    it('dispatch a heartbeat', async function () {
        const [Alice, Bob] = await Promise.all([generateNode(), generateNode()]);
        await Promise.all([
            new Promise(resolve => {
                Bob.network.heartbeat.once('beat', (peerId) => {
                    assert_1.default(peerId.isEqual(Alice.peerInfo.id), 'connection must come from Alice');
                    resolve();
                });
            }),
            Alice.interactions.network.heartbeat.interact(Bob.peerInfo)
        ]);
        await Promise.all([Alice.stop(), Bob.stop()]);
    });
});
