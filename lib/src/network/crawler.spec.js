"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
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
const interactions_1 = require("../interactions");
const crawler_1 = require("./crawler");
const crawler_2 = require("../interactions/network/crawler");
const multiaddr_1 = __importDefault(require("multiaddr"));
describe('test crawler', function () {
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
        node.peerRouting.findPeer = (_) => Promise.reject('not implemented');
        node.interactions = {
            network: {
                crawler: new crawler_2.Crawler(node)
            }
        };
        new interactions_1.Interactions(node);
        node.network = {
            crawler: new crawler_1.Crawler(node)
        };
        node.log = debug_1.default(`${chalk_1.default.blue(node.peerInfo.id.toB58String())}: `);
        return node;
    }
    it('should crawl the network and find some nodes', async function () {
        const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([generateNode(), generateNode(), generateNode(), generateNode(), generateNode()]);
        await assert_1.default.rejects(() => Alice.network.crawler.crawl(), Error(`Unable to find enough other nodes in the network.`));
        Alice.peerStore.put(Bob.peerInfo);
        await assert_1.default.rejects(() => Alice.network.crawler.crawl(), Error(`Unable to find enough other nodes in the network.`));
        Bob.peerStore.put(Chris.peerInfo);
        await assert_1.default.rejects(() => Alice.network.crawler.crawl(), Error(`Unable to find enough other nodes in the network.`));
        Chris.peerStore.put(Dave.peerInfo);
        await assert_1.default.doesNotReject(() => Alice.network.crawler.crawl(), `Should find enough nodes.`);
        Bob.peerStore.put(Alice.peerInfo);
        Dave.peerStore.put(Eve.peerInfo);
        await assert_1.default.doesNotReject(() => Bob.network.crawler.crawl(), `Should find enough nodes.`);
        await Promise.all([
            /* prettier-ignore */
            Alice.stop(),
            Bob.stop(),
            Chris.stop(),
            Dave.stop(),
            Eve.stop()
        ]);
    });
});
