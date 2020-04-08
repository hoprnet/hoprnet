"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const Multiaddr = require('multiaddr');
const _1 = require(".");
describe('test peerInfo serialisation', function () {
    it('should serialize and deserilize a peerInfo', async function () {
        const peerInfo = await peer_info_1.default.create(await peer_id_1.default.create({ keyType: 'secp256k1' }));
        assert_1.default((await _1.deserializePeerInfo(Buffer.from(_1.serializePeerInfo(peerInfo)))).id.toB58String() == peerInfo.id.toB58String(), `Serialized peerInfo should be deserializable and id should match.`);
        const testMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/0');
        peerInfo.multiaddrs.add(testMultiaddr);
        assert_1.default((await _1.deserializePeerInfo(Buffer.from(_1.serializePeerInfo(peerInfo)))).multiaddrs.has(testMultiaddr), `Serialized peerInfo should be deserializable and multiaddrs should match.`);
        const secondTestMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/1');
        peerInfo.multiaddrs.add(secondTestMultiaddr);
        const thirdTestMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/2');
        const deserialized = await _1.deserializePeerInfo(Buffer.from(_1.serializePeerInfo(peerInfo)));
        assert_1.default(deserialized.multiaddrs.has(testMultiaddr) && deserialized.multiaddrs.has(secondTestMultiaddr) && !deserialized.multiaddrs.has(thirdTestMultiaddr), `Serialized peerInfo should be deserializable and multiaddrs should match.`);
    });
});
